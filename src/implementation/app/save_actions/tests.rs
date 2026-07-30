use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::document::ActiveDocument;
use crate::file_io;

use super::{save_document, save_document_as};

static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

#[test]
fn ui_save_paths_delegate_to_the_production_save_service() {
    let source = include_str!("../save_actions.rs");

    assert!(source.contains("save_service::save_document(document)"));
    assert!(source.contains("save_service::save_document_as(document, path, overwrite)"));
    assert!(!source.contains("file_io::save_markdown"));
    assert!(!source.contains("document.replace_file_path"));
}

#[test]
fn ui_save_as_preserves_source_and_transitions_shared_identity() {
    let root = test_root("save-as");
    let source = root.join("source.md");
    let target = root.join("saved-as.md");
    let mut document = writable_document(&source);
    document.update_markdown("# Current memory".to_string());
    let version_before = document.version();

    let saved_path = save_document_as(&mut document, &target, false).unwrap();

    assert_eq!(saved_path, target.canonicalize().unwrap());
    assert_eq!(fs::read_to_string(&source).unwrap(), "# Original");
    assert_eq!(fs::read_to_string(&target).unwrap(), "# Current memory");
    assert_eq!(document.canonical_path(), saved_path);
    assert!(document.version() > version_before);
    assert!(!document.is_dirty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ui_save_observes_shared_external_change_gate() {
    let root = test_root("external");
    let source = root.join("source.md");
    let mut document = writable_document(&source);
    document.update_markdown("# Current memory".to_string());
    fs::write(&source, "# External").unwrap();

    let failure = save_document(&mut document).unwrap_err();

    assert!(failure.contains("changed outside MarkHola"));
    assert_eq!(fs::read_to_string(&source).unwrap(), "# External");
    assert!(document.is_dirty());
    fs::remove_dir_all(root).unwrap();
}

fn test_root(label: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "markhola-ui-save-{label}-{}-{}",
        std::process::id(),
        NEXT_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn writable_document(source: &std::path::Path) -> ActiveDocument {
    fs::write(source, "# Original").unwrap();
    let mut document = ActiveDocument::open_with_id(
        17,
        source.to_path_buf(),
        "# Original".to_string(),
        file_io::directory_base_url(source).unwrap(),
    );
    document.toggle_mode();
    document
}
