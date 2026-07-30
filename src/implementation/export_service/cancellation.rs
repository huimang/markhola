use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use super::ExportError;

static CANCELLATIONS: OnceLock<Mutex<CancellationRegistry>> = OnceLock::new();

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

    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Default)]
struct CancellationRegistry {
    active: HashMap<String, ExportCancellation>,
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
    let mut registry = registry().lock().unwrap_or_else(|error| error.into_inner());
    if let Some(cancellation) = registry.active.get(request_id) {
        return cancellation.clone();
    }
    let cancellation = ExportCancellation::default();
    registry
        .active
        .insert(request_id.to_string(), cancellation.clone());
    cancellation
}

pub(crate) fn finish_export_cancellation(request_id: &str) {
    registry()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .active
        .remove(request_id);
}

pub(crate) fn request_export_cancellation(request_id: &str) -> bool {
    let registry = registry().lock().unwrap_or_else(|error| error.into_inner());
    if let Some(cancellation) = registry.active.get(request_id) {
        cancellation.cancel();
        true
    } else {
        false
    }
}

fn registry() -> &'static Mutex<CancellationRegistry> {
    CANCELLATIONS.get_or_init(|| Mutex::new(CancellationRegistry::default()))
}
