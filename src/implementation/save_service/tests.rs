use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::document::{ActiveDocument, DocumentMode};

use super::{save_document, save_document_as, validate_save_as_target};

static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

fn root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "markhola-save-service-{label}-{}-{}",
        std::process::id(),
        NEXT_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn writable_document(path: &Path, markdown: &str) -> ActiveDocument {
    fs::write(path, markdown).unwrap();
    let mut document = ActiveDocument::open_with_id(
        1,
        path.to_path_buf(),
        markdown.to_string(),
        crate::file_io::directory_base_url(path).unwrap(),
    );
    assert_eq!(document.mode(), DocumentMode::Readonly);
    document.toggle_mode();
    document
}

#[test]
fn saves_writable_document_and_rejects_external_change() {
    let root = root("existing");
    let path = root.join("source.md");
    let mut document = writable_document(&path, "# Original");
    document.update_markdown("# Current memory".to_string());
    let version = document.version();

    assert_eq!(
        save_document(&mut document).unwrap(),
        path.canonicalize().unwrap()
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "# Current memory");
    assert!(document.version() > version);
    fs::write(&path, "# External").unwrap();
    document.update_markdown("# Next edit".to_string());
    assert_eq!(
        save_document(&mut document).unwrap_err().code,
        "external_source_changed"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "# External");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_as_preserves_source_and_transitions_only_after_atomic_success() {
    let root = root("save-as");
    let source = root.join("source.md");
    let target = root.join("copy.md");
    let mut document = writable_document(&source, "# Original");
    document.update_markdown("# Unsaved current memory".to_string());

    assert_eq!(
        save_document_as(&mut document, &target, false).unwrap(),
        target
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .join("copy.md")
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "# Original");
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "# Unsaved current memory"
    );
    assert_eq!(document.canonical_path(), target.canonicalize().unwrap());
    assert!(!document.is_dirty());

    let existing = root.join("existing.md");
    fs::write(&existing, "keep").unwrap();
    let before = document.canonical_path().to_path_buf();
    assert_eq!(
        save_document_as(&mut document, &existing, false)
            .unwrap_err()
            .code,
        "output_exists"
    );
    assert_eq!(document.canonical_path(), before);
    assert_eq!(fs::read_to_string(existing).unwrap(), "keep");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_relative_wrong_extension_and_symlink_targets() {
    let root = root("paths");
    assert_eq!(
        validate_save_as_target(Path::new("relative.md"), false)
            .unwrap_err()
            .code,
        "invalid_output_path"
    );
    assert_eq!(
        validate_save_as_target(&root.join("wrong.txt"), false)
            .unwrap_err()
            .code,
        "invalid_output_extension"
    );
    let source = root.join("source.md");
    let link = root.join("link.md");
    fs::write(&source, "source").unwrap();
    std::os::unix::fs::symlink(&source, &link).unwrap();
    assert_eq!(
        validate_save_as_target(&link, true).unwrap_err().code,
        "unsafe_output_path"
    );
    fs::remove_dir_all(root).unwrap();
}
