use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::document::ActiveDocument;

use super::output::validate_target;
use super::{
    ExportCancellation, ExportFormat, begin_export_cancellation, export_document_to_path,
    finish_export_cancellation, request_export_cancellation,
};

static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

fn test_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "markhola-export-service-{label}-{}-{}",
        std::process::id(),
        NEXT_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn registered_queued_export_applies_cancellation_cooperatively() {
    let request_id = format!(
        "queued-cancel-{}",
        NEXT_TEST.fetch_add(1, Ordering::Relaxed)
    );
    begin_export_cancellation(&request_id);
    assert!(request_export_cancellation(&request_id));
    let cancellation = begin_export_cancellation(&request_id);
    assert_eq!(cancellation.check().unwrap_err().code, "cancelled");
    finish_export_cancellation(&request_id);
}

fn document(root: &Path) -> ActiveDocument {
    let path = root.join("source.md");
    let markdown = "# Current memory\n\nUnsaved body.".to_string();
    ActiveDocument::open_with_id(1, path, markdown, format!("file://{}/", root.display()))
}

#[test]
fn validates_absolute_format_specific_safe_targets() {
    let root = test_root("paths");
    assert_eq!(
        validate_target(Path::new("relative.html"), ExportFormat::Html, false)
            .unwrap_err()
            .code,
        "invalid_output_path"
    );
    assert_eq!(
        validate_target(&root.join("wrong.pdf"), ExportFormat::Html, false)
            .unwrap_err()
            .code,
        "invalid_output_extension"
    );
    fs::write(root.join("exists.html"), "old").unwrap();
    assert_eq!(
        validate_target(&root.join("exists.html"), ExportFormat::Html, false)
            .unwrap_err()
            .code,
        "output_exists"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn html_export_commits_atomically_and_preserves_current_memory() {
    let root = test_root("html");
    let document = document(&root);
    let output = root.join("document.html");
    let result = export_document_to_path(
        &document,
        ExportFormat::Html,
        &output,
        false,
        &ExportCancellation::default(),
    )
    .unwrap();

    let html = fs::read_to_string(&output).unwrap();
    assert!(html.contains("Current memory"));
    assert!(html.contains("Unsaved body."));
    assert_eq!(
        result.path,
        root.canonicalize().unwrap().join("document.html")
    );
    assert_eq!(result.sha256.len(), 64);
    assert!(fs::read_dir(&root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp")
    }));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cancellation_never_replaces_existing_output() {
    let root = test_root("cancel");
    let output = root.join("document.html");
    fs::write(&output, "original").unwrap();
    let cancellation = ExportCancellation::default();
    cancellation.cancel();
    let failure = export_document_to_path(
        &document(&root),
        ExportFormat::Html,
        &output,
        true,
        &cancellation,
    )
    .unwrap_err();

    assert_eq!(failure.code, "cancelled");
    assert_eq!(fs::read_to_string(&output).unwrap(), "original");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_local_image_fails_without_creating_output() {
    let root = test_root("missing-asset");
    let source = root.join("source.md");
    let document = ActiveDocument::open_with_id(
        1,
        source,
        "![missing](./missing.png)".to_string(),
        format!("file://{}/", root.display()),
    );
    let output = root.join("document.html");
    let failure = export_document_to_path(
        &document,
        ExportFormat::Html,
        &output,
        false,
        &ExportCancellation::default(),
    )
    .unwrap_err();

    assert_eq!(failure.code, "missing_local_asset");
    assert!(!output.exists());
    fs::remove_dir_all(root).unwrap();
}
