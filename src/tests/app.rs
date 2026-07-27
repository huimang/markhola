use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

use crate::app::{AppLanguage, AppTheme, DocumentSize};
use crate::workspace::DocumentWorkspace;

use super::implementation::{
    file_paths_from_urls, load_document, markdown_path_from_href,
    reload_workspace_documents_from_disk,
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
    let html = app_shell_html(
        AppTheme::Default,
        AppLanguage::English,
        DocumentSize::default(),
    );

    assert!(html.contains("id=\"findPanel\""));
    assert!(html.contains("data:image/png;base64,"));
    assert!(html.contains("<img class=\"about-logo\""));
    assert!(html.contains("window.openFindPanel = openFindPanel;"));
    assert!(html.contains("window.applyAppTheme = applyAppTheme;"));
    assert!(html.contains("window.applyAppLanguage = applyAppLanguage;"));
    assert!(html.contains("id=\"appThemeStyle\""));
    assert!(html.contains("className = \"find-match\""));
    assert!(html.contains("replaceAllWritableMatches"));
    assert!(html.contains("event.key.toLowerCase() === \"f\""));
    assert!(!html.contains("class=\"bottom-bar\""));
}

#[test]
fn app_shell_embeds_the_requested_language_catalog() {
    let english = app_shell_html(
        AppTheme::Default,
        AppLanguage::English,
        DocumentSize::default(),
    );
    let chinese = app_shell_html(
        AppTheme::Default,
        AppLanguage::SimplifiedChinese,
        DocumentSize::default(),
    );

    assert!(english.contains("A free Markdown reader built with AI."));
    assert!(chinese.contains("基于 AI 构建的免费 Markdown 阅读器。"));
    assert!(chinese.contains("\"language\":\"zh-CN\""));
    assert!(!english.contains("__APP_LANGUAGE__"));
    assert!(!chinese.contains("__APP_LANGUAGE__"));
}

#[test]
fn app_shell_uses_requested_theme_css() {
    let html = app_shell_html(
        AppTheme::Light,
        AppLanguage::English,
        DocumentSize::default(),
    );

    assert!(html.contains("#eef3f8"));
    assert!(html.contains("id=\"appThemeStyle\""));
}

#[test]
fn app_themes_share_the_markhola_brand_palette() {
    for theme in AppTheme::ALL {
        let html = app_shell_html(theme, AppLanguage::English, DocumentSize::default());

        for declaration in [
            "--markhola-violet: #6657E8",
            "--markhola-violet-mid: #ACA4F4",
            "--markhola-violet-tint: #F2F0FF",
            "--markhola-violet-subtle: #F9F8FF",
            "--markhola-green: #17B890",
            "--markhola-green-mid: #81D9C3",
            "--markhola-green-tint: #EAF9F5",
            "--markhola-green-subtle: #FAFEFD",
            "--markhola-gray-strong: #1E293B",
            "--markhola-gray: #475569",
            "--markhola-gray-mid: #94A3B8",
            "--markhola-gray-tint: #E2E8F0",
            "--markhola-gray-subtle: #F8FAFC",
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
    let html = app_shell_html(
        AppTheme::Dark,
        AppLanguage::English,
        DocumentSize::default(),
    );

    assert!(html.contains(
        ".markdown-body table {\n  width: 100%;\n  border-collapse: collapse;\n  background: var(--panel-strong);"
    ));
    assert!(html.contains(
        ".markdown-body thead {\n  background: var(--markhola-green-tint);\n  color: var(--table-header-text);"
    ));
}

#[test]
fn default_theme_uses_green_subtle_for_its_background_surfaces() {
    let html = app_shell_html(
        AppTheme::Default,
        AppLanguage::English,
        DocumentSize::default(),
    );

    assert!(html.contains("--bg: var(--markhola-green-subtle)"));
    assert!(html.contains("--panel: rgba(250, 254, 253, 0.88)"));
    assert!(html.contains("--panel-strong: rgba(250, 254, 253, 0.97)"));
    assert!(html.contains("background: var(--bg)"));
    assert!(!html.contains("--bg: #f5f1e8"));
    assert!(!html.contains("rgba(255, 251, 243"));
}

#[test]
fn default_theme_separates_titlebar_tabs_and_document_surface() {
    let native_tabs = include_str!("../implementation/app/native_tabs.rs");
    let html = app_shell_html(
        AppTheme::Default,
        AppLanguage::English,
        DocumentSize::default(),
    );

    assert!(
        native_tabs
            .contains("AppTheme::Default => ((234, 249, 245), (226, 232, 240), 0.34)")
    );
    assert!(
        native_tabs.contains("AppTheme::Light => ((234, 249, 245), (226, 232, 240), 0.34)")
    );
    assert!(native_tabs.contains("ns_window.setBackgroundColor(Some(&tab_color))"));
    assert!(native_tabs.contains("NSWindowButton::CloseButton"));
    assert!(native_tabs.contains("titlebar_effect_view(ns_window)"));
    assert!(native_tabs.contains("TITLEBAR_BACKGROUND_IDENTIFIER"));
    assert!(native_tabs.contains("NSVisualEffectMaterial::Titlebar"));
    assert!(native_tabs.contains("NSVisualEffectBlendingMode::WithinWindow"));
    assert!(html.contains("--bg: var(--markhola-green-subtle)"));
    assert!(html.contains("--markhola-gray-tint: #E2E8F0"));
    assert!(html.contains("--markhola-green-tint: #EAF9F5"));
    assert!(html.contains("--markhola-green-subtle: #FAFEFD"));
}

#[test]
fn app_themes_use_the_semantic_danger_color_for_mermaid_errors() {
    for theme in AppTheme::ALL {
        let html = app_shell_html(theme, AppLanguage::English, DocumentSize::default());

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
    let html = app_shell_html(
        AppTheme::Default,
        AppLanguage::English,
        DocumentSize::from_stored(130),
    );

    assert!(html.contains("window.applyDocumentSize = applyDocumentSize;"));
    assert!(html.contains("applyDocumentSize(130);"));
    assert!(!html.contains("__DOCUMENT_SIZE__"));
}

#[test]
fn app_shell_leaves_tabs_to_appkit_and_keeps_outline_controls() {
    let html = app_shell_html(
        AppTheme::Default,
        AppLanguage::English,
        DocumentSize::default(),
    );

    assert!(!html.contains("id=\"tabsBar\""));
    assert!(!html.contains("class=\"tabs-shell"));
    assert!(!html.contains("id=\"previousTabs\""));
    assert!(!html.contains("id=\"nextTabs\""));
    assert!(!html.contains("id=\"newDocumentTab\""));
    assert!(html.contains("id=\"outlinePanel\""));
    assert!(html.contains("id=\"outlineClose\""));
    assert!(!html.contains("document-toolbar"));
    assert!(html.contains("body.workspace-empty .empty-state"));
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
    assert!(html.contains("const refreshOutline = () =>"));
    assert!(html.contains("heading.scrollIntoView"));
    assert!(html.contains("window.setOutlinePanelOpen = (open) =>"));
    assert!(html.contains(r#"{ kind: "toggle-outline" }"#));
    assert!(html.contains("background: var(--markhola-violet-tint)"));
    assert!(!html.contains("Readonly preview updated."));
    assert!(!html.contains("Writable mode enabled."));
    assert!(!html.contains("brand-mark"));
    assert!(!html.contains("more-button"));
    assert!(!html.contains("file-dot"));
}

#[test]
fn native_footer_contains_document_controls_in_the_confirmed_order() {
    let source = include_str!("../implementation/app/native_footer.rs");
    let controls = ["path_field:", "words_field:", "lines_field:", "mode_field:"];
    let positions =
        controls.map(|control| source.find(control).expect("footer control should exist"));

    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(!source.contains("NSButton"));
    assert!(!source.contains("DocumentSize"));
    assert!(!source.contains("\"Mode "));
    assert!(!source.contains("\"Status "));
    assert!(source.contains("set_label_text(&handle.path_field, &active.file_path)"));
    assert!(source.contains("set_label_text(&handle.mode_field, mode)"));
    assert!(!source.contains("status_field"));
}

#[test]
fn view_menu_owns_the_checked_outline_toggle() {
    let menu = include_str!("../implementation/app/menu_view.rs");
    let state = include_str!("../implementation/app/menu_state.rs");

    assert!(menu.contains("text(\"menu.outline\")"));
    assert!(menu.contains("sel!(toggleOutlinePanel:)"));
    assert!(menu.contains("remember_outline_item(&outline_item)"));
    assert!(state.contains("toggle_outline_selected"));
    assert!(state.contains("set_outline_available"));
}

#[test]
fn app_shell_removes_failed_mermaid_render_artifacts() {
    let html = app_shell_html(
        AppTheme::Default,
        AppLanguage::English,
        DocumentSize::default(),
    );

    assert!(html.contains("const removeMermaidRenderArtifact = (renderId) =>"));
    assert!(html.contains("document.getElementById(`d${renderId}`)?.remove()"));
    assert!(html.contains("removeMermaidRenderArtifact(renderId)"));
}

#[test]
fn native_footer_uses_the_compact_v080_height() {
    let source = include_str!("../implementation/app/native_footer.rs");

    assert!(source.contains("const FOOTER_HEIGHT: f64 = 34.0;"));
    assert!(source.contains("const FOOTER_LABEL_Y: f64 = 7.0;"));
    assert!(!source.contains("const FOOTER_HEIGHT: f64 = 38.0;"));
    assert!(!source.contains("const FOOTER_HEIGHT: f64 = 42.0;"));
}

#[test]
fn native_footer_uses_one_lower_brightness_text_color() {
    let source = include_str!("../implementation/app/native_footer.rs");

    for field in ["path_field", "words_field", "lines_field", "mode_field"] {
        assert!(source.contains(&format!("handle.{field}.setTextColor(Some(&foreground))")));
    }
    assert!(!source.contains("setTextColor(Some(&primary))"));
    assert!(!source.contains("setTextColor(Some(&secondary))"));
    assert!(source.contains("rgb_color(71, 85, 105)"));
    assert!(source.contains("rgb_color(148, 163, 184)"));
}

#[test]
fn native_footer_right_aligns_metadata_but_keeps_the_path_on_the_left() {
    let source = include_str!("../implementation/app/native_footer.rs");

    for field in ["words_field", "lines_field", "mode_field"] {
        assert!(source.contains(&format!(
            "{field}.setAlignment(NSTextAlignment::Right)"
        )));
    }
    assert!(!source.contains("path_field.setAlignment(NSTextAlignment::Right)"));
}

#[test]
fn native_footer_omits_the_status_block() {
    let source = include_str!("../implementation/app/native_footer.rs");

    assert!(!source.contains("FOOTER_STATUS_WIDTH"));
    assert!(!source.contains("status_field"));
    assert!(!source.contains("footer.saved"));
    assert!(!source.contains("footer.unsaved"));
}

#[test]
fn app_theme_keys_are_stable() {
    let summary = AppTheme::ALL
        .iter()
        .map(|theme| theme.key())
        .collect::<Vec<_>>();

    assert_eq!(summary, vec!["default", "dark", "light"]);
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

#[test]
fn toggle_mode_lives_only_in_edit_menu_source() {
    let file_menu = include_str!("../implementation/app/menu_file.rs");
    let edit_menu = include_str!("../implementation/app/menu_edit.rs");

    assert!(!file_menu.contains("toggleDocumentMode:"));
    assert!(edit_menu.contains("toggleDocumentMode:"));
}

#[test]
fn native_tabs_keep_appkit_tab_controls_and_numeric_shortcuts() {
    let menu_install = include_str!("../implementation/app/menu_install.rs");
    let menu_tab = include_str!("../implementation/app/menu_tab.rs");
    let native_tabs = include_str!("../implementation/app/native_tabs.rs");

    assert!(!menu_install.contains("toggleTabBar:"));
    assert!(!menu_install.contains("toggleTabOverview:"));
    assert!(menu_tab.contains("activateDocumentOne:"));
    assert!(menu_tab.contains("activateDocumentNine:"));
    assert!(native_tabs.contains("tabbedWindows()"));
    assert!(native_tabs.contains("NSWindowTabbingMode::Preferred"));
}

#[test]
fn markdown_path_from_href_accepts_file_markdown_urls_and_ignores_others() {
    assert_eq!(
        markdown_path_from_href("file:///tmp/hello.md"),
        Some(PathBuf::from("/tmp/hello.md"))
    );
    assert_eq!(
        markdown_path_from_href("file:///tmp/guide.markdown#intro"),
        Some(PathBuf::from("/tmp/guide.markdown"))
    );
    assert_eq!(
        markdown_path_from_href("https://example.com/hello.md"),
        None
    );
    assert_eq!(markdown_path_from_href("file:///tmp/image.png"), None);
    assert_eq!(markdown_path_from_href("not a url"), None);
}

#[test]
fn app_shell_routes_local_markdown_links_through_ipc() {
    let script = include_str!("../implementation/app/shell_script.js");

    assert!(script.contains("const resolveLocalMarkdownLink = (href) =>"));
    assert!(script.contains("decodeURIComponent(fileUrl.pathname || \"\").toLowerCase()"));
    assert!(script.contains("fileUrl.protocol !== \"file:\""));
    assert!(script.contains("pathname.endsWith(\".md\")"));
    assert!(script.contains("pathname.endsWith(\".markdown\")"));
    assert!(script.contains("kind: \"open-markdown-link\""));
}
