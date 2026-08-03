use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::document::ActiveDocument;
use crate::file_io::load_markdown;
use crate::html_export::export_markdown_file_to_path;

use super::build_export_html;
use super::build_export_html_with_theme_and_context;
use crate::app::DocumentSize;
use crate::pdf_export::RenderContext;

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("markhola-html-export-{name}-{stamp}.{extension}"))
}

#[test]
fn exported_html_contains_rendered_content_and_runtime_assets() {
    let document = ActiveDocument::open_with_id(
        1,
        PathBuf::from("/tmp/demo.md"),
        "# Hello\n\n```mermaid\nflowchart TD\n A-->B\n```".to_string(),
        "file:///tmp/".to_string(),
    );

    let html = build_export_html(&document);

    assert!(html.contains("<article class=\"markdown-body\">"));
    assert!(html.contains("mermaid-block"));
    assert!(html.contains("Exported by MarkHola v"));
    assert!(html.contains("window.MathJax"));
    assert!(html.contains("window.mermaid"));
    assert!(html.contains("document.getElementById(`d${renderId}`)?.remove()"));
    assert!(html.contains("removeMermaidRenderArtifact(renderId)"));
    assert!(html.contains("const normalizeMermaidSourceForRender = (source) =>"));
    assert!(html.contains(r#"source.replaceAll("\\n", "<br/>")"#));
    assert!(html.contains("window.mermaid.render(renderId, renderSource)"));
    assert!(html.contains("escapeHtml(source)"));
    assert!(html.contains("overflow: auto;"));
}

#[test]
fn standalone_html_preserves_the_accessible_scrollable_table_region() {
    let document = ActiveDocument::open_with_id(
        2,
        PathBuf::from("/tmp/table.md"),
        "| A | B |\n| - | - |\n| long | content |".to_string(),
        "file:///tmp/".to_string(),
    );

    let html = build_export_html(&document);
    assert!(html.contains("class=\"markdown-table-region\""));
    assert!(html.contains("role=\"region\""));
    assert!(html.contains("tabindex=\"0\""));
    assert!(html.contains("overflow-x: auto;"));
    assert!(html.contains("body:not(.static-table-export) .markdown-body tbody tr:hover"));
    assert!(html.contains("--table-row-odd:"));
    assert!(html.contains("--table-row-even:"));
    assert!(!html.contains("<body class=\"static-table-export\">"));
}

#[test]
fn standalone_html_uses_the_shared_accessible_footnote_model() {
    let document = ActiveDocument::open_with_id(
        1,
        PathBuf::from("/tmp/footnotes.md"),
        "Shared[^note] and again[^note].\n\n[^note]: Export footnote.".to_string(),
        "file:///tmp/".to_string(),
    );

    let html = build_export_html(&document);

    assert!(html.contains("class=\"footnotes\" aria-label=\"Footnotes\""));
    assert!(html.contains("id=\"markhola-footnote-ref-1-2\""));
    assert!(html.contains("href=\"#markhola-footnote-ref-1-2\""));
    assert!(html.contains("Export footnote."));
}

#[test]
fn export_markdown_file_to_path_preserves_source_and_writes_html_document() {
    let input = temp_path("source", "md");
    let output = temp_path("export", "html");
    let markdown = "# Export Title\n\n```mermaid\nflowchart TD\n A-->B\n```";
    std::fs::write(&input, markdown).unwrap();

    export_markdown_file_to_path(&input, &output).unwrap();

    let html = std::fs::read_to_string(&output).unwrap();
    let expected_title = input.file_name().unwrap().to_string_lossy();
    assert_eq!(load_markdown(&input).unwrap(), markdown);
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<base href=\"file://"));
    assert!(html.contains(&format!("<title>{expected_title}</title>")));
    assert!(html.contains("Exported by MarkHola v"));
    assert!(html.contains("window.mermaid.render(renderId, renderSource)"));
    assert!(html.contains("window.MathJax"));

    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&output);
}

#[test]
fn export_verification_example_html_keeps_local_asset_and_full_render_features() {
    let source = include_str!("../../examples/v0.9.1-png-export-and-save-as.md");
    let document = ActiveDocument::open_with_id(
        9,
        PathBuf::from("/tmp/v0.9.1-png-export-and-save-as.md"),
        source.to_string(),
        "file:///tmp/examples/".to_string(),
    );

    let html = build_export_html(&document);

    assert!(html.contains("./assets/diagram.svg"));
    assert!(html.contains("window.mermaid.render(renderId, renderSource)"));
    assert!(html.contains("window.MathJax"));
    assert!(html.contains("temporary export verification"));
    assert!(html.contains("Save As preserves the original copied source"));
    assert!(html.contains("Light expectation"));
    assert!(html.contains("Dark expectation"));
    assert!(html.contains("full-document export"));
}

#[test]
fn html_export_context_applies_fifty_one_hundred_and_two_hundred_percent_typography() {
    let document = ActiveDocument::open_with_id(
        10,
        PathBuf::from("/tmp/context.md"),
        "# Heading\n\nBody".to_string(),
        "file:///tmp/".to_string(),
    );
    for (percent, body, heading, code) in [
        (50, "8.5px", "18.4px", "7px"),
        (100, "17px", "36.8px", "14px"),
        (200, "34px", "73.6px", "28px"),
    ] {
        let html = build_export_html_with_theme_and_context(
            &document,
            "default",
            RenderContext::new(DocumentSize::from_stored(percent)),
        );
        assert!(html.contains(&format!("--document-font-size: {body}")));
        assert!(html.contains(&format!("--document-h1-font-size: {heading}")));
        assert!(html.contains(&format!("--document-code-font-size: {code}")));
    }
}
