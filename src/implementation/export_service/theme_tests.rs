use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app::AppTheme;
use crate::document::ActiveDocument;

use super::{ExportCancellation, ExportFormat, export_document_to_path_with_theme};

static NEXT_THEME_TEST: AtomicU64 = AtomicU64::new(1);

#[test]
fn html_export_uses_the_exact_active_theme() {
    let root = std::env::temp_dir().join(format!(
        "markhola-theme-export-{}-{}",
        std::process::id(),
        NEXT_THEME_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    let document = ActiveDocument::open_with_id(
        1,
        root.join("source.md"),
        "# Theme parity\n\n```rust\nlet value = 1;\n```".to_string(),
        format!("file://{}/", root.display()),
    );

    let light = export_document_to_path_with_theme(
        &document,
        AppTheme::Default,
        ExportFormat::Html,
        &root.join("light.html"),
        false,
        &ExportCancellation::default(),
    )
    .unwrap();
    let dark = export_document_to_path_with_theme(
        &document,
        AppTheme::Dark,
        ExportFormat::Html,
        &root.join("dark.html"),
        false,
        &ExportCancellation::default(),
    )
    .unwrap();
    let light_html = fs::read_to_string(light.path).unwrap();
    let dark_html = fs::read_to_string(dark.path).unwrap();

    assert_ne!(light.sha256, dark.sha256);
    assert_ne!(light_html, dark_html);
    assert!(light_html.contains("theme: \"default\""));
    assert!(dark_html.contains("theme: \"dark\""));
    assert!(light_html.contains("Theme parity"));
    assert!(dark_html.contains("Theme parity"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ui_and_protocol_delegate_theme_to_the_same_export_service() {
    let ui = include_str!("../app/export_actions.rs");
    let protocol = include_str!("../app/protocol_commands/mod.rs");
    let events = include_str!("../app/user_events.rs");

    assert!(ui.contains("export_service::export_document_to_path_with_theme("));
    assert!(protocol.contains("export_service::export_document_to_path_with_theme("));
    assert!(events.contains("runtime.selected_theme"));
    assert!(events.contains("handle_app_mut_with_theme("));
    assert!(!ui.contains("render_document_png_data"));
}

#[test]
fn png_and_pdf_share_theme_specific_render_input() {
    let root = std::env::temp_dir().join(format!(
        "markhola-theme-render-input-{}-{}",
        std::process::id(),
        NEXT_THEME_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("local.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="#17B890"/></svg>"##,
    )
    .unwrap();
    let document = ActiveDocument::open_with_id(
        2,
        root.join("source.md"),
        "```mermaid\ngraph TD\nA-->B\n```\n\n$E=mc^2$\n\n![local](./local.svg)".to_string(),
        format!("file://{}/", root.display()),
    );
    let rendered = crate::pdf_export::render_export_document_html(&document);
    let light = crate::pdf_export::build_export_html_with_theme(&document, &rendered, "default");
    let dark = crate::pdf_export::build_export_html_with_theme(&document, &rendered, "dark");

    assert_ne!(light, dark);
    assert!(light.contains("theme: \"default\""));
    assert!(dark.contains("theme: \"dark\""));
    assert!(light.contains("background: var(--bg)"));
    assert!(dark.contains("background: var(--bg)"));
    assert!(light.contains("color: var(--text)"));
    assert!(dark.contains("color: var(--text)"));
    assert!(!light.contains("background: #ffffff"));
    assert!(!dark.contains("background: #ffffff"));
    assert!(!light.contains("color: #111111"));
    assert!(!dark.contains("color: #111111"));
    assert!(light.contains("graph TD"));
    assert!(dark.contains("graph TD"));
    assert!(light.contains("math math-inline"));
    assert!(dark.contains("math math-inline"));
    assert!(light.contains("data:image/svg+xml;base64,"));
    assert!(dark.contains("data:image/svg+xml;base64,"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn shared_export_service_preserves_fixture_source_and_theme_parity() {
    let root = std::env::temp_dir().join(format!(
        "markhola-cli-export-fixture-{}-{}",
        std::process::id(),
        NEXT_THEME_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("fixture.md");
    let source_markdown = include_str!("../../tests/fixtures/v0.9.2-offline-cli-export.md");
    fs::write(&source_path, source_markdown).unwrap();
    fs::write(
        root.join("local-diagram.svg"),
        include_str!("../../tests/fixtures/local-diagram.svg"),
    )
    .unwrap();
    let document = ActiveDocument::open_with_id(
        3,
        source_path.clone(),
        source_markdown.to_string(),
        format!("file://{}/", root.display()),
    );

    let source_before = fs::read_to_string(&source_path).unwrap();
    let light = export_document_to_path_with_theme(
        &document,
        AppTheme::Default,
        ExportFormat::Html,
        &root.join("light.html"),
        false,
        &ExportCancellation::default(),
    )
    .unwrap();
    let dark = export_document_to_path_with_theme(
        &document,
        AppTheme::Dark,
        ExportFormat::Html,
        &root.join("dark.html"),
        false,
        &ExportCancellation::default(),
    )
    .unwrap();
    let source_after = fs::read_to_string(&source_path).unwrap();
    let light_html = fs::read_to_string(light.path).unwrap();
    let dark_html = fs::read_to_string(dark.path).unwrap();

    assert_eq!(source_before, source_after);
    assert_ne!(light.sha256, dark.sha256);
    assert!(light_html.contains("theme: \"default\""));
    assert!(dark_html.contains("theme: \"dark\""));
    assert!(light_html.contains("Offline CLI Export Fixture"));
    assert!(dark_html.contains("Offline CLI Export Fixture"));
    assert!(light_html.contains("flowchart TD"));
    assert!(dark_html.contains("flowchart TD"));
    assert!(light_html.contains("math math-inline"));
    assert!(dark_html.contains("math math-inline"));
    assert!(light_html.contains("math math-display"));
    assert!(dark_html.contains("math math-display"));
    assert!(light_html.contains("data:image/svg+xml;base64,"));
    assert!(dark_html.contains("data:image/svg+xml;base64,"));
    assert!(light_html.contains("Paragraph 5"));
    assert!(dark_html.contains("Paragraph 5"));
    fs::remove_dir_all(root).unwrap();
}
