use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::ExportError;

const LIFECYCLE_LIMIT: usize = 256;

static CANCELLATIONS: OnceLock<(Mutex<CancellationRegistry>, Condvar)> = OnceLock::new();

thread_local! {
    static CURRENT: RefCell<Option<ExportCancellation>> = const { RefCell::new(None) };
}

#[derive(Clone, Default)]
pub struct ExportCancellation(Arc<AtomicBool>);

impl ExportCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub(super) fn check(&self) -> Result<(), ExportError> {
        if self.is_cancelled() {
            Err(ExportError::new(
                "cancelled",
                "The export request was cancelled.",
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Default)]
struct CancellationRegistry {
    entries: HashMap<String, LifecycleEntry>,
    order: VecDeque<String>,
}

struct LifecycleEntry {
    cancellation: ExportCancellation,
    status: ExportStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExportStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ExportStatus {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CancelOutcome {
    Cancelled,
    TooLate,
    NotFound,
    TimedOut,
}

pub(super) fn run_with_cancellation<T>(
    cancellation: &ExportCancellation,
    operation: impl FnOnce() -> T,
) -> T {
    CURRENT.with(|slot| slot.replace(Some(cancellation.clone())));
    let result = operation();
    CURRENT.with(|slot| slot.replace(None));
    result
}

pub(crate) fn cooperative_cancellation_requested() -> bool {
    CURRENT.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(ExportCancellation::is_cancelled)
    })
}

pub(crate) fn begin_export_cancellation(request_id: &str) -> ExportCancellation {
    let (registry, changed) = registry();
    let mut registry = registry.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(entry) = registry.entries.get_mut(request_id) {
        entry.status = ExportStatus::Running;
        changed.notify_all();
        return entry.cancellation.clone();
    }
    let cancellation = ExportCancellation::default();
    insert_entry(
        &mut registry,
        request_id,
        cancellation.clone(),
        ExportStatus::Running,
    );
    changed.notify_all();
    cancellation
}

pub(crate) fn register_queued_export(request_id: &str) {
    let (registry, changed) = registry();
    let mut registry = registry.lock().unwrap_or_else(|error| error.into_inner());
    if !registry.entries.contains_key(request_id) {
        insert_entry(
            &mut registry,
            request_id,
            ExportCancellation::default(),
            ExportStatus::Queued,
        );
        changed.notify_all();
    }
}

pub(crate) fn finish_export(request_id: &str, status: ExportStatus) {
    debug_assert!(status.is_terminal());
    let (registry, changed) = registry();
    let mut registry = registry.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(entry) = registry.entries.get_mut(request_id) {
        entry.status = status;
        changed.notify_all();
    }
}

pub(crate) fn finish_unresolved_export(request_id: &str) {
    let (registry, changed) = registry();
    let mut registry = registry.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(entry) = registry.entries.get_mut(request_id)
        && !entry.status.is_terminal()
    {
        entry.status = ExportStatus::Failed;
        changed.notify_all();
    }
}

#[cfg(test)]
pub(crate) fn request_export_cancellation(request_id: &str) -> bool {
    let (registry, _) = registry();
    let registry = registry.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(entry) = registry.entries.get(request_id) {
        entry.cancellation.cancel();
        true
    } else {
        false
    }
}

#[cfg(test)]
pub(crate) fn finish_export_cancellation(request_id: &str) {
    let (registry, changed) = registry();
    registry
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .entries
        .remove(request_id);
    changed.notify_all();
}

pub(crate) fn export_status(request_id: &str) -> Option<ExportStatus> {
    registry()
        .0
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .entries
        .get(request_id)
        .map(|entry| entry.status)
}

pub(crate) fn cancel_export_and_wait(request_id: &str, timeout: Duration) -> CancelOutcome {
    let (registry, changed) = registry();
    let mut registry = registry.lock().unwrap_or_else(|error| error.into_inner());
    let Some(entry) = registry.entries.get(request_id) else {
        return CancelOutcome::NotFound;
    };
    if entry.status.is_terminal() {
        return terminal_cancel_outcome(entry.status);
    }
    entry.cancellation.cancel();

    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return CancelOutcome::TimedOut;
        }
        let (next, wait) = changed
            .wait_timeout(registry, remaining)
            .unwrap_or_else(|error| error.into_inner());
        registry = next;
        let Some(entry) = registry.entries.get(request_id) else {
            return CancelOutcome::NotFound;
        };
        if entry.status.is_terminal() {
            return terminal_cancel_outcome(entry.status);
        }
        if wait.timed_out() {
            return CancelOutcome::TimedOut;
        }
    }
}

fn terminal_cancel_outcome(status: ExportStatus) -> CancelOutcome {
    if status == ExportStatus::Cancelled {
        CancelOutcome::Cancelled
    } else {
        CancelOutcome::TooLate
    }
}

fn insert_entry(
    registry: &mut CancellationRegistry,
    request_id: &str,
    cancellation: ExportCancellation,
    status: ExportStatus,
) {
    while registry.entries.len() >= LIFECYCLE_LIMIT {
        let candidates = registry.order.len();
        let mut removed = false;
        for _ in 0..candidates {
            let Some(candidate) = registry.order.pop_front() else {
                break;
            };
            if registry
                .entries
                .get(&candidate)
                .is_some_and(|entry| entry.status.is_terminal())
            {
                registry.entries.remove(&candidate);
                removed = true;
                break;
            }
            registry.order.push_back(candidate);
        }
        if !removed {
            break;
        }
    }
    registry.order.push_back(request_id.to_string());
    registry.entries.insert(
        request_id.to_string(),
        LifecycleEntry {
            cancellation,
            status,
        },
    );
}

fn registry() -> &'static (Mutex<CancellationRegistry>, Condvar) {
    CANCELLATIONS.get_or_init(|| (Mutex::new(CancellationRegistry::default()), Condvar::new()))
}
