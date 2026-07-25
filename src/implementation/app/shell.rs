mod shell_markup;
mod shell_script;

use std::sync::atomic::{AtomicBool, Ordering};

use crate::app::{AppTheme, DocumentSize};
use crate::render_assets;

use self::shell_markup::APP_SHELL_MARKUP;
use self::shell_script::APP_SHELL_SCRIPT;

const WORKSPACE_CHROME_CSS: &str = include_str!("workspace_chrome.css");

pub(crate) fn app_shell_html(theme: AppTheme, document_size: DocumentSize) -> String {
    [APP_SHELL_MARKUP, APP_SHELL_SCRIPT]
        .join("")
        .replace(
            "__APP_THEME__",
            &render_assets::load_app_theme_css_for_inline_style(theme.key()),
        )
        .replace("__WORKSPACE_CHROME__", WORKSPACE_CHROME_CSS)
        .replace(
            "__MERMAID_RUNTIME__",
            &render_assets::mermaid_runtime_for_inline_script(),
        )
        .replace(
            "__MATHJAX_RUNTIME__",
            &render_assets::mathjax_runtime_for_inline_script(),
        )
        .replace("__DOCUMENT_SIZE__", &document_size.percent().to_string())
        .replace("__APP_ICON_DATA_URL__", render_assets::app_icon_data_url())
        .replace("__APP_LOGO_DATA_URL__", render_assets::app_logo_data_url())
}

pub(crate) fn should_recover_shell_on_page_load(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.is_empty() || trimmed == "about:blank"
}

pub(crate) fn should_dispatch_shell_recovery(url: &str, suppress_once: &AtomicBool) -> bool {
    should_recover_shell_on_page_load(url) && !suppress_once.swap(false, Ordering::SeqCst)
}
