use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

use crate::app::{AppTheme, DocumentSize};
use crate::workspace::DocumentWorkspace;

use super::implementation::{
    file_paths_from_urls, load_document, reload_workspace_documents_from_disk,
};
use super::shell::{
    app_shell_html, should_dispatch_shell_recovery, should_recover_shell_on_page_load,
};

fn temp_markdown_path(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("markhola-reload-{name}-{stamp}.md"))
}

#[test]
fn reload_workspace_refreshes_active_document_from_disk() {
    let path = temp_markdown_path("reload");
    fs::write(&path, "# Before\nold content").unwrap();

    let mut workspace = DocumentWorkspace::new();
    let document = load_document(1, &path).unwrap();
    workspace.open_document(document);

    fs::write(&path, "# After\nnew content").unwrap();

    let status = reload_workspace_documents_from_disk(&mut workspace).unwrap();
    let snapshot = workspace.active_document_snapshot().unwrap();

    assert_eq!(status, "Document reloaded.");
    assert_eq!(snapshot.markdown, "# After\nnew content");
    assert!(snapshot.html.contains("After"));
    assert!(snapshot.html.contains("new content"));
    assert!(!snapshot.dirty);

    let _ = fs::remove_file(path);
}

#[test]
fn recovers_shell_when_page_load_finishes_on_blank_url() {
    assert!(should_recover_shell_on_page_load("about:blank"));
    assert!(should_recover_shell_on_page_load(""));
    assert!(!should_recover_shell_on_page_load("file:///tmp/demo.md"));
    assert!(!should_recover_shell_on_page_load("data:text/html,hello"));
}

#[test]
fn suppresses_the_expected_blank_finish_once_before_recovering_again() {
    let suppress_once = AtomicBool::new(true);

    assert!(!should_dispatch_shell_recovery(
        "about:blank",
        &suppress_once
    ));
    assert!(should_dispatch_shell_recovery(
        "about:blank",
        &suppress_once
    ));
    assert!(!should_dispatch_shell_recovery(
        "file:///tmp/demo.md",
        &suppress_once
    ));
}

#[test]
fn app_shell_includes_find_panel_markup_and_handlers() {
    let html = app_shell_html(AppTheme::Default, DocumentSize::default());

    assert!(html.contains("id=\"findPanel\""));
    assert!(html.contains("data:image/png;base64,"));
    assert!(html.contains("<img class=\"about-logo\""));
    assert!(html.contains("window.openFindPanel = openFindPanel;"));
    assert!(html.contains("window.applyAppTheme = applyAppTheme;"));
    assert!(html.contains("id=\"appThemeStyle\""));
    assert!(html.contains("className = \"find-match\""));
    assert!(html.contains("replaceAllWritableMatches"));
    assert!(html.contains("event.key.toLowerCase() === \"f\""));
    assert!(!html.contains("class=\"bottom-bar\""));
}

#[test]
fn app_shell_uses_requested_theme_css() {
    let html = app_shell_html(AppTheme::Light, DocumentSize::default());

    assert!(html.contains("#eef3f8"));
    assert!(html.contains("id=\"appThemeStyle\""));
}

#[test]
fn app_themes_share_the_markhola_brand_palette() {
    for theme in AppTheme::ALL {
        let html = app_shell_html(theme, DocumentSize::default());

        for declaration in [
            "--markhola-violet: #6657E8",
            "--markhola-violet-mid: #ACA4F4",
            "--markhola-violet-tint: #F2F0FF",
            "--markhola-green: #17B890",
            "--markhola-green-mid: #81D9C3",
            "--markhola-green-tint: #EAF9F5",
            "--accent: var(--markhola-violet)",
            "--accent-strong: var(--markhola-green)",
        ] {
            assert!(
                html.contains(declaration),
                "{} theme should include {declaration}",
                theme.key()
            );
        }

        assert!(html.contains(".markdown-body thead"));
        assert!(html.contains("background: var(--markhola-green-tint)"));
        assert!(html.contains("--code-surface: #24213a"));
        assert!(html.contains("background: var(--code-gutter)"));
        assert!(html.contains("color: var(--code-line-number)"));
        assert!(!html.contains("--brand-violet"));
        assert!(!html.contains("--brand-green"));
    }
}

#[test]
fn dark_theme_keeps_table_rows_readable_below_the_green_header() {
    let html = app_shell_html(AppTheme::Dark, DocumentSize::default());

    assert!(html.contains(
        ".markdown-body table {\n  width: 100%;\n  border-collapse: collapse;\n  background: var(--panel-strong);"
    ));
    assert!(html.contains(
        ".markdown-body thead {\n  background: var(--markhola-green-tint);\n  color: var(--table-header-text);"
    ));
}

#[test]
fn app_themes_use_the_semantic_danger_color_for_mermaid_errors() {
    for theme in AppTheme::ALL {
        let html = app_shell_html(theme, DocumentSize::default());

        assert!(html.contains(
            ".markdown-body .mermaid-block__error {\n  margin: 0;\n  padding: 12px 16px 16px;\n  color: var(--danger);"
        ));
    }
}

#[test]
fn document_size_uses_expected_steps_and_limits() {
    let default_size = DocumentSize::default();
    assert_eq!(default_size.percent(), 100);
    assert_eq!(default_size.increase().percent(), 110);
    assert_eq!(default_size.decrease().percent(), 90);

    let mut maximum = default_size;
    let mut minimum = default_size;
    for _ in 0..20 {
        maximum = maximum.increase();
        minimum = minimum.decrease();
    }
    assert_eq!(maximum.percent(), 200);
    assert_eq!(minimum.percent(), 50);
}

#[test]
fn document_size_restores_only_supported_values() {
    assert_eq!(DocumentSize::from_stored(130).percent(), 130);
    assert_eq!(DocumentSize::from_stored(135).percent(), 100);
    assert_eq!(DocumentSize::from_stored(49).percent(), 100);
    assert_eq!(DocumentSize::from_stored(201).percent(), 100);
    assert_eq!(DocumentSize::from_stored(-1).percent(), 100);
}

#[test]
fn app_shell_includes_restored_document_size() {
    let html = app_shell_html(AppTheme::Default, DocumentSize::from_stored(130));

    assert!(html.contains("window.applyDocumentSize = applyDocumentSize;"));
    assert!(html.contains("applyDocumentSize(130);"));
    assert!(!html.contains("__DOCUMENT_SIZE__"));
}

#[test]
fn app_shell_includes_workspace_tab_and_outline_controls() {
    let html = app_shell_html(AppTheme::Default, DocumentSize::default());

    assert!(html.contains("id=\"tabsBar\""));
    assert!(html.contains("class=\"tabs-shell hidden\""));
    assert!(html.contains(".tabs-shell {\n  min-width: 0;\n  min-height: 36px"));
    assert!(html.contains("id=\"previousTabs\""));
    assert!(html.contains("id=\"nextTabs\""));
    assert!(html.contains("id=\"newDocumentTab\""));
    assert!(html.contains("id=\"outlinePanel\""));
    assert!(html.contains("id=\"outlineClose\""));
    assert!(!html.contains("document-toolbar"));
    assert!(html.contains("tabsShell.classList.toggle(\"hidden\", !tabs.length)"));
    assert!(html.contains("body.workspace-empty .empty-state"));
    assert!(html.contains("body.workspace-empty .app"));
    assert!(html.contains("body.workspace-empty .preview-shell"));
    assert!(html.contains("class=\"empty-state__icon\""));
    assert!(html.contains("alt=\"MarkHola\""));
    assert!(html.contains(
        "body.workspace-empty .empty-card {\n  display: grid;\n  grid-template-columns: 96px minmax(0, 1fr);"
    ));
    assert!(html.contains(".empty-state__icon {\n  display: block;\n  width: 96px;"));
    assert!(html.contains("object-fit: contain"));
    assert!(html.contains("grid-template-rows: minmax(0, 1fr)"));
    assert!(html.contains("place-items: center"));
    assert!(!html.contains("id=\"currentPath\""));
    assert!(!html.contains("<span>Documents</span>"));
    assert!(!html.contains("id=\"modeValue\""));
    assert!(!html.contains("id=\"decreaseSize\""));
    assert!(!html.contains("id=\"sizeValue\""));
    assert!(!html.contains("id=\"increaseSize\""));
    assert!(html.contains("kind: \"new-document\""));
    assert!(html.contains("const refreshOutline = () =>"));
    assert!(html.contains("heading.scrollIntoView"));
    assert!(html.contains("window.setOutlinePanelOpen = (open) =>"));
    assert!(html.contains(r#"{ kind: "toggle-outline" }"#));
    assert!(html.contains(".document-tab.active::after"));
    assert!(html.contains("background: var(--markhola-green)"));
    assert!(html.contains("background: var(--markhola-violet-tint)"));
    assert!(html.contains("border-color: var(--markhola-green-mid)"));
    assert!(!html.contains("Readonly preview updated."));
    assert!(!html.contains("Writable mode enabled."));
    assert!(!html.contains("brand-mark"));
    assert!(!html.contains("more-button"));
    assert!(!html.contains("file-dot"));
}

#[test]
fn native_footer_contains_document_controls_in_the_confirmed_order() {
    let source = include_str!("../implementation/app/native_footer.rs");
    let controls = [
        "path_field:",
        "words_field:",
        "lines_field:",
        "mode_field:",
        "status_field:",
    ];
    let positions = controls
        .map(|control| source.find(control).expect("footer control should exist"));

    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(!source.contains("NSButton"));
    assert!(!source.contains("DocumentSize"));
    assert!(!source.contains("\"Mode "));
    assert!(!source.contains("\"Status "));
    assert!(source.contains("set_label_text(&handle.path_field, &active.file_path)"));
    assert!(source.contains("set_label_text(&handle.mode_field, &active.mode_label)"));
    assert!(source.contains("set_label_text(&handle.status_field, status)"));
}

#[test]
fn view_menu_owns_the_checked_outline_toggle() {
    let menu = include_str!("../implementation/app/menu_view.rs");
    let state = include_str!("../implementation/app/menu_state.rs");

    assert!(menu.contains("\"Outline\""));
    assert!(menu.contains("sel!(toggleOutlinePanel:)"));
    assert!(menu.contains("remember_outline_item(&outline_item)"));
    assert!(state.contains("toggle_outline_selected"));
    assert!(state.contains("set_outline_available"));
}

#[test]
fn app_shell_removes_failed_mermaid_render_artifacts() {
    let html = app_shell_html(AppTheme::Default, DocumentSize::default());

    assert!(html.contains("const removeMermaidRenderArtifact = (renderId) =>"));
    assert!(html.contains("document.getElementById(`d${renderId}`)?.remove()"));
    assert!(html.contains("removeMermaidRenderArtifact(renderId)"));
}

#[test]
fn app_theme_keys_and_labels_are_stable() {
    let summary = AppTheme::ALL
        .iter()
        .map(|theme| (theme.key(), theme.label()))
        .collect::<Vec<_>>();

    assert_eq!(
        summary,
        vec![
            ("default", "Default"),
            ("dark", "Dark"),
            ("light", "Light"),
        ]
    );
}

#[test]
fn app_theme_round_trips_from_stable_key() {
    for theme in AppTheme::ALL {
        assert_eq!(AppTheme::from_key(theme.key()), Some(theme));
    }

    assert_eq!(AppTheme::from_key("unknown"), None);
    assert_eq!(AppTheme::from_key("github"), None);
}

#[test]
fn file_paths_from_urls_keeps_all_file_paths_in_order() {
    let paths = file_paths_from_urls(vec![
        Url::parse("file:///tmp/one.md").unwrap(),
        Url::parse("file:///tmp/two.md").unwrap(),
        Url::parse("file:///tmp/three.md").unwrap(),
    ]);

    assert_eq!(
        paths,
        vec![
            PathBuf::from("/tmp/one.md"),
            PathBuf::from("/tmp/two.md"),
            PathBuf::from("/tmp/three.md"),
        ]
    );
}

#[test]
fn file_paths_from_urls_ignores_non_file_urls() {
    let paths = file_paths_from_urls(vec![
        Url::parse("https://example.com/demo.md").unwrap(),
        Url::parse("file:///tmp/real.md").unwrap(),
    ]);

    assert_eq!(paths, vec![PathBuf::from("/tmp/real.md")]);
}
