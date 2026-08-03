use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::DocumentSize;
use crate::document::{ActiveDocument, suggested_pdf_export_path};
use crate::markdown;
use lopdf::{Dictionary, Document as LoDocument, Object, Stream};

use super::implementation::{
    APP_NAME, APP_VERSION, EXPORT_WEBVIEW_HEIGHT, EXPORT_WEBVIEW_WIDTH, ExportMeasurement,
    ExportPreparationMode, MIN_STATIC_TABLE_SCALE, PDF_CANVAS_WIDTH, PDF_READING_SURFACE_WIDTH,
    STATIC_TABLE_CONTENT_WIDTH, apply_pdf_metadata, build_export_html,
    build_export_html_with_theme_and_context, export_capture_rect, export_footer_text,
    export_preparation_mode, RenderContext,
    printable_page_count_for_height,
};

fn document(path: &str, markdown: &str) -> ActiveDocument {
    ActiveDocument::open_with_id(
        1,
        PathBuf::from(path),
        markdown.to_string(),
        "file:///tmp/".to_string(),
    )
}

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("markhola-pdf-export-{name}-{stamp}.{extension}"))
}

#[test]
fn suggested_pdf_path_replaces_markdown_extension() {
    assert_eq!(
        suggested_pdf_export_path(PathBuf::from("/tmp/note.md").as_path()),
        PathBuf::from("/tmp/note.pdf")
    );
    assert_eq!(
        suggested_pdf_export_path(PathBuf::from("/tmp/README.markdown").as_path()),
        PathBuf::from("/tmp/README.pdf")
    );
    assert_eq!(
        suggested_pdf_export_path(PathBuf::from("/tmp/notes").as_path()),
        PathBuf::from("/tmp/notes.pdf")
    );
}

#[test]
fn suggested_pdf_path_keeps_source_document_path_unchanged() {
    let document = document("/tmp/source-preserved.md", "# Source");

    let export_path = document.suggested_pdf_export_path();

    assert_eq!(
        document.file_path(),
        PathBuf::from("/tmp/source-preserved.md")
    );
    assert_eq!(export_path, PathBuf::from("/tmp/source-preserved.pdf"));
}

#[test]
fn export_html_contains_document_content_without_app_shell() {
    let document = document(
        "/tmp/example.md",
        "# Example\n\n```mermaid\nflowchart TD\n  A-->B\n```\n\n$$x^2$$",
    );
    let html = build_export_html(&document, &markdown::render_html(document.markdown()));

    assert!(html.contains("<main class=\"export-reading-surface\">"));
    assert!(html.contains("markdown-body export-page"));
    assert!(html.contains(".export-reading-surface {"));
    assert!(html.contains("max-width: 960px;"));
    assert!(html.contains("grid-template-columns: minmax(0, 960px);"));
    assert!(html.contains("justify-content: center;"));
    assert!(!html.contains("__PDF_READING_SURFACE_WIDTH__"));
    assert!(html.contains("window.markholaPreparePdf"));
    assert!(html.contains("mermaid-block"));
    assert!(html.contains("math math-display"));
    assert!(html.contains("document.getElementById(`d${renderId}`)?.remove()"));
    assert!(html.contains("removeMermaidRenderArtifact(renderId)"));
    assert!(html.contains("const normalizeMermaidSourceForRender = (source) =>"));
    assert!(html.contains(r#"source.replaceAll("\\n", "<br/>")"#));
    assert!(html.contains("window.mermaid.render(renderId, renderSource)"));
    assert!(html.contains("escapeHtml(source)"));
    assert!(html.contains(&export_footer_text()));
    assert!(!html.contains("<div class=\"tabs-bar\""));
    assert!(!html.contains("<div class=\"editor-pane\""));
    assert!(!html.contains("<div class=\"about-overlay\""));
}

#[test]
fn static_exports_fit_only_tables_to_the_frozen_content_width() {
    let document = document(
        "/tmp/wide.md",
        "| A | B |\n| - | - |\n| long | content |",
    );
    let html = build_export_html(&document, &markdown::render_html(document.markdown()));

    assert_eq!(STATIC_TABLE_CONTENT_WIDTH, 848.0);
    assert_eq!(MIN_STATIC_TABLE_SCALE, 0.75);
    assert!(html.contains("<body class=\"static-table-export\">"));
    assert!(html.contains("const contentWidth = 848;"));
    assert!(html.contains("const minimumScale = 0.75;"));
    assert!(html.contains("region.scrollLeft = 0;"));
    assert!(html.contains("table.style.zoom = String(scale);"));
    assert!(html.contains("table_too_wide:"));
    assert!(html.contains(".static-table-export .markdown-table-region::-webkit-scrollbar"));
    assert!(html.contains("body:not(.static-table-export) .markdown-body tbody tr:hover"));
    assert!(html.contains("@media print"));
    assert!(!html.contains("__STATIC_TABLE_CONTENT_WIDTH__"));
    assert!(!html.contains("__MIN_STATIC_TABLE_SCALE__"));
}

#[test]
fn png_pdf_and_print_share_complete_accessible_footnote_html() {
    let document = document(
        "/tmp/footnotes.md",
        "Body[^long] and repeated[^long].\n\n[^long]: A long shared-output footnote with **formatting**, $x^2$, and enough content to remain part of the measured full document height.",
    );
    let rendered = markdown::render_html(document.markdown());
    let light = build_export_html_with_theme_and_context(
        &document,
        &rendered,
        "default",
        RenderContext::default(),
    );
    let dark = build_export_html_with_theme_and_context(
        &document,
        &rendered,
        "dark",
        RenderContext::default(),
    );

    for html in [&rendered, &light, &dark] {
        assert!(html.contains("class=\"footnotes\" aria-label=\"Footnotes\""));
        assert!(html.contains("id=\"markhola-footnote-ref-1-2\""));
        assert!(html.contains("href=\"#markhola-footnote-ref-1-2\""));
        assert!(html.contains("A long shared-output footnote"));
        assert!(html.contains("<strong>formatting</strong>"));
        assert!(html.contains("class=\"math math-inline\""));
    }
    assert!(light.find("class=\"footnotes\"").unwrap() < light.find("class=\"export-footer\"").unwrap());
    assert!(dark.find("class=\"footnotes\"").unwrap() < dark.find("class=\"export-footer\"").unwrap());

    let source = include_str!("../implementation/pdf_export.rs");
    assert!(source.contains("render_document_png_data_with_theme"));
    assert!(source.contains("render_document_pdf_data_with_theme"));
    assert!(source.contains("prepare_printable_webview_with_context"));
    assert!(source.contains("markdown::render_html_with_image_resolver"));
    assert!(source.contains("height: measurement.height"));
}

#[test]
fn print_preparation_does_not_enable_static_table_fitting() {
    let source = include_str!("../implementation/pdf_export.rs");
    assert!(source.contains(
        "prepare_webview_with_measurement_with_theme_and_context(document, theme_name, context, false)"
    ));
    assert!(source.contains(
        "prepare_webview_with_measurement_with_theme_and_context(document, theme_name, context, true)"
    ));
}

#[test]
fn export_markdown_file_to_path_rejects_missing_source_without_creating_output() {
    let input = temp_path("missing", "md");
    let output = temp_path("missing-output", "pdf");

    let error = crate::pdf_export::export_markdown_file_to_path(&input, &output).unwrap_err();

    assert!(error.contains("Failed to canonicalize input path"));
    assert!(!output.exists());
}

#[test]
fn uses_fast_prepare_for_plain_markdown() {
    let document = document("/tmp/plain.md", "# Plain\n\nhello world");
    assert_eq!(
        export_preparation_mode(&markdown::render_html(document.markdown())),
        ExportPreparationMode::Fast
    );
}

#[test]
fn uses_full_prepare_for_async_rendering_content() {
    let document = document(
        "/tmp/async.md",
        "# Async\n\n```mermaid\nflowchart TD\n  A-->B\n```\n\n![img](./demo.png)\n\n$E=mc^2$",
    );
    assert_eq!(
        export_preparation_mode(&markdown::render_html(document.markdown())),
        ExportPreparationMode::Full
    );
}

#[test]
fn injects_pdf_metadata_with_markhola_version() {
    let document = document("/tmp/meta.md", "# Meta");
    let mut base = LoDocument::with_version("1.5");
    let pages_id = base.new_object_id();
    let page_id = base.new_object_id();
    let content_id = base.add_object(Stream::new(Dictionary::new(), Vec::new()));
    let resources_id = base.add_object(Dictionary::new());

    let mut page = Dictionary::new();
    page.set("Type", "Page");
    page.set("Parent", pages_id);
    page.set("Contents", content_id);
    page.set("Resources", resources_id);
    page.set(
        "MediaBox",
        vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(100),
            Object::Integer(100),
        ],
    );
    base.objects.insert(page_id, Object::Dictionary(page));

    let mut pages = Dictionary::new();
    pages.set("Type", "Pages");
    pages.set("Kids", vec![page_id.into()]);
    pages.set("Count", 1);
    base.objects.insert(pages_id, Object::Dictionary(pages));

    let mut catalog = Dictionary::new();
    catalog.set("Type", "Catalog");
    catalog.set("Pages", pages_id);
    let catalog_id = base.add_object(catalog);
    base.trailer.set("Root", catalog_id);

    let mut base_pdf = Vec::new();
    base.save_to(&mut base_pdf)
        .expect("base pdf should serialize");

    let output =
        apply_pdf_metadata(&document, base_pdf).expect("metadata injection should succeed");
    let parsed = LoDocument::load_mem(&output).expect("output pdf should parse");
    let info_ref = parsed
        .trailer
        .get(b"Info")
        .expect("info should exist")
        .as_reference()
        .expect("info should be a reference");
    let info = parsed
        .get_dictionary(info_ref)
        .expect("info dictionary should be readable");

    assert_eq!(
        info.get(b"Creator").and_then(Object::as_str).ok(),
        Some(APP_NAME.as_bytes())
    );
    assert_eq!(
        info.get(b"Producer").and_then(Object::as_str).ok(),
        Some(format!("{APP_NAME} v{APP_VERSION}").as_bytes())
    );
}

#[test]
fn export_capture_rect_uses_full_measured_height() {
    let rect = export_capture_rect(&ExportMeasurement {
        width: EXPORT_WEBVIEW_WIDTH,
        height: 4820.0,
        error: None,
    });

    assert_eq!(rect.size.width, PDF_CANVAS_WIDTH);
    assert_eq!(rect.size.height, 4820.0);
}

#[test]
fn export_capture_rect_keeps_fixed_canvas_width_for_wide_content() {
    let rect = export_capture_rect(&ExportMeasurement {
        width: 2048.0,
        height: 4820.0,
        error: None,
    });

    assert_eq!(rect.size.width, PDF_CANVAS_WIDTH);
    assert_eq!(rect.size.height, 4820.0);
}

#[test]
fn export_capture_rect_respects_minimum_viewport_size() {
    let rect = export_capture_rect(&ExportMeasurement {
        width: 100.0,
        height: 200.0,
        error: None,
    });

    assert_eq!(rect.size.width, PDF_CANVAS_WIDTH);
    assert_eq!(rect.size.height, EXPORT_WEBVIEW_HEIGHT);
}

#[test]
fn pdf_canvas_has_symmetric_thirty_two_pixel_gutters() {
    assert_eq!(PDF_CANVAS_WIDTH, 1024.0);
    assert_eq!(PDF_READING_SURFACE_WIDTH, 960.0);
    assert_eq!((PDF_CANVAS_WIDTH - PDF_READING_SURFACE_WIDTH) / 2.0, 32.0);
}

#[test]
fn print_render_context_snapshots_document_size_without_changing_pdf_geometry() {
    let document = ActiveDocument::new_blank_with_id(1, 1);
    for (percent, body, heading, code) in [
        (50, "8.5px", "18.4px", "7px"),
        (100, "17px", "36.8px", "14px"),
        (200, "34px", "73.6px", "28px"),
    ] {
        let context = RenderContext::new(DocumentSize::from_stored(percent));
        let html = build_export_html_with_theme_and_context(
            &document,
            "<p>Body</p>",
            "default",
            context,
        );
        assert!(html.contains(&format!("--document-font-size: {body}")));
        assert!(html.contains(&format!("--document-h1-font-size: {heading}")));
        assert!(html.contains(&format!("--document-code-font-size: {code}")));
        assert!(html.contains("grid-template-columns: minmax(0, 960px)"));
    }
}

#[test]
fn printable_page_count_rounds_up_for_partial_last_page() {
    assert_eq!(printable_page_count_for_height(EXPORT_WEBVIEW_HEIGHT), 1);
    assert_eq!(
        printable_page_count_for_height(EXPORT_WEBVIEW_HEIGHT + 1.0),
        2
    );
    assert_eq!(printable_page_count_for_height(6647.0), 6);
}

#[test]
#[ignore = "Requires WKWebView JavaScript evaluation support (may fail in sandboxed/headless environments)."]
fn mermaid_example_print_preview_generates_expected_page_count() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let example_path = root_dir.join("examples/mermaid.md");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("markhola")
        .arg("--")
        .arg("--smoke-print-pages")
        .arg(&example_path)
        .current_dir(&root_dir)
        .output()
        .expect("smoke print page-count command should start");

    assert!(
        output.status.success(),
        "smoke print page-count failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let page_count = stdout
        .split("pages=")
        .nth(1)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .expect("stdout should contain the computed print page count");

    assert_eq!(page_count, 6, "examples/mermaid.md page count changed");
}
