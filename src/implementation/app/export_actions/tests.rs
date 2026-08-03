use std::path::Path;

use super::suggested_png_path;

#[test]
fn png_ui_uses_format_specific_path_and_shared_export_service() {
    assert_eq!(
        suggested_png_path(Path::new("/tmp/document.markdown")),
        Path::new("/tmp/document.png")
    );
    let source = include_str!("../export_actions.rs");
    assert!(source.contains("export_service::export_document_to_path_with_theme_and_context("));
    assert!(source.contains("ExportFormat::Png"));
    assert!(source.contains("ExportCancellation::default()"));
    assert!(source.contains(".add_filter(\"PNG\", &[\"png\"])"));
    assert!(source.contains(".set_title(text(\"dialog.export_png\"))"));
    assert!(source.contains(".set_directory("));
    assert!(source.contains(".set_file_name(suggested_name)"));
    assert!(source.contains("let Some(path) = choose_png_export_path(document) else {"));
    assert!(source.contains("&path,\n        true,"));
    assert!(source.contains(
        "&text(\"status.exported_png\").replace(\"{path}\", &result.path.display().to_string())"
    ));
    assert!(
        source
            .contains("text(\"status.export_png_failed\").replace(\"{error}\", &failure.message)")
    );
    assert!(!source.contains("render_document_png_data"));
    assert!(!source.contains("File::create("));
    assert!(!source.contains("std::fs::write("));
}

#[test]
fn ui_exports_snapshot_document_size_and_share_context_aware_service() {
    let events = include_str!("../user_events.rs");
    let actions = include_str!("../export_actions.rs");
    let service = include_str!("../../export_service.rs");

    for event in ["ExportPng", "ExportPdf", "ExportHtml"] {
        let branch = events
            .split(&format!("UserEvent::{event} =>"))
            .nth(1)
            .unwrap_or_else(|| panic!("missing {event} branch"));
        let branch = branch.split("UserEvent::").next().unwrap();
        assert!(branch.contains("RenderContext::new(runtime.document_size)"));
    }
    assert_eq!(
        actions
            .matches("export_document_to_path_with_theme_and_context(")
            .count(),
        3
    );
    assert!(!actions.contains("pageZoom"));
    assert!(!actions.contains("pinch"));
    assert!(service.contains("RenderContext::default(),\n        format,"));
}
