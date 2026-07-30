use std::path::Path;

use super::suggested_png_path;

#[test]
fn png_ui_uses_format_specific_path_and_shared_export_service() {
    assert_eq!(
        suggested_png_path(Path::new("/tmp/document.markdown")),
        Path::new("/tmp/document.png")
    );
    let source = include_str!("../export_actions.rs");
    assert!(source.contains("export_service::export_document_to_path("));
    assert!(source.contains("ExportFormat::Png"));
    assert!(source.contains("ExportCancellation::default()"));
    assert!(source.contains(".add_filter(\"PNG\", &[\"png\"])"));
    assert!(!source.contains("render_document_png_data"));
}
