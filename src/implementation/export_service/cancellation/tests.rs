use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use super::{
    CancelOutcome, CancellationRegistry, ExportCancellation, ExportStatus, LIFECYCLE_LIMIT,
    cancel_export_and_wait, export_status, finish_export_cancellation, insert_entry,
    register_queued_export,
};

const REGISTRY_LOCK: &str = "/tmp/markhola-export-registry-test-lock";
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

struct RegistryGuard;

impl RegistryGuard {
    fn acquire() -> Self {
        loop {
            match fs::create_dir(REGISTRY_LOCK) {
                Ok(()) => return Self,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("failed to acquire export registry test lock: {error}"),
            }
        }
    }
}

impl Drop for RegistryGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir(REGISTRY_LOCK);
    }
}

fn request_id(label: &str) -> String {
    format!(
        "{label}-{}-{}",
        std::process::id(),
        NEXT_REQUEST.fetch_add(1, Ordering::Relaxed),
    )
}

#[test]
fn rejects_new_registration_when_all_capacity_entries_are_active() {
    let mut registry = CancellationRegistry::default();
    for index in 0..LIFECYCLE_LIMIT {
        let status = if index % 2 == 0 {
            ExportStatus::Queued
        } else {
            ExportStatus::Running
        };
        assert!(insert_entry(
            &mut registry,
            &format!("active-{index}"),
            ExportCancellation::default(),
            status,
        ));
    }

    assert!(!insert_entry(
        &mut registry,
        "overflow",
        ExportCancellation::default(),
        ExportStatus::Queued,
    ));
    assert_eq!(registry.entries.len(), LIFECYCLE_LIMIT);
    assert_eq!(registry.order.len(), LIFECYCLE_LIMIT);
    assert!(!registry.entries.contains_key("overflow"));

    let active = registry.entries.get("active-0").unwrap();
    active.cancellation.cancel();
    assert!(active.cancellation.is_cancelled());
    assert_eq!(active.status, ExportStatus::Queued);

    registry.entries.get_mut("active-0").unwrap().status = ExportStatus::Completed;
    assert!(insert_entry(
        &mut registry,
        "replacement",
        ExportCancellation::default(),
        ExportStatus::Queued,
    ));
    assert_eq!(registry.entries.len(), LIFECYCLE_LIMIT);
    assert!(!registry.entries.contains_key("active-0"));
    assert!(registry.entries.contains_key("replacement"));
}

#[test]
fn public_capacity_registration_keeps_active_entries_queryable_until_terminal_eviction() {
    let _guard = RegistryGuard::acquire();
    let ids = (0..LIFECYCLE_LIMIT)
        .map(|index| request_id(&format!("public-capacity-{index}")))
        .collect::<Vec<_>>();

    for (index, request_id) in ids.iter().enumerate() {
        assert!(register_queued_export(request_id));
        if index % 2 == 1 {
            super::begin_export_cancellation(request_id);
        }
    }

    let overflow = request_id("public-capacity-overflow");
    assert!(!register_queued_export(&overflow));
    assert!(export_status(&overflow).is_none());
    assert_eq!(export_status(&ids[0]), Some(ExportStatus::Queued));
    assert_eq!(export_status(&ids[1]), Some(ExportStatus::Running));
    assert_eq!(
        cancel_export_and_wait(&ids[1], Duration::from_millis(1)),
        CancelOutcome::TimedOut
    );
    assert_eq!(export_status(&ids[1]), Some(ExportStatus::Running));

    super::finish_export(&ids[0], ExportStatus::Completed);
    assert!(register_queued_export(&overflow));
    assert_eq!(export_status(&overflow), Some(ExportStatus::Queued));
    assert_eq!(export_status(&ids[0]), None);
    for request_id in ids.iter().skip(1) {
        finish_export_cancellation(request_id);
    }
    finish_export_cancellation(&overflow);
}
