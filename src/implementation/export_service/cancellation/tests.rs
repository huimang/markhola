use super::{
    CancellationRegistry, ExportCancellation, ExportStatus, LIFECYCLE_LIMIT, insert_entry,
};

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
