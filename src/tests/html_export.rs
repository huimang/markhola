use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::document::ActiveDocument;
use crate::html_export::export_markdown_file_to_path;
use crate::file_io::load_markdown;

use super::build_export_html;

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
