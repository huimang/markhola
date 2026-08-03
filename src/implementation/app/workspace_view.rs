use tao::window::Window;
use wry::WebView;

use crate::app::{AppLanguage, ThemePreference, text};
use crate::workspace::DocumentWorkspace;

use super::native_footer::NativeFooter;
use super::native_tabs;
use super::{
    APP_AUTHOR, APP_BUILD_PLATFORM, APP_BUILD_TARGET, APP_GITHUB_URL, APP_VERSION,
    ErrorStatusPayload, ExportSuccessPayload, WINDOW_TITLE, WorkspacePresentation, macos_menu,
};

pub(super) fn present_workspace(
    window: &Window,
    webview: &WebView,
    native_footer: &NativeFooter,
    workspace: &DocumentWorkspace,
    status: &str,
    full_render: bool,
) {
    update_window_title(window, workspace.active_window_title().as_deref());
    sync_native_window_state(window, workspace);
    sync_native_menu_state(workspace);
    native_footer.sync(workspace, status);
    if full_render {
        render_workspace(webview, workspace, status);
    } else {
        sync_workspace_state(window, webview, native_footer, workspace, status);
    }
}

pub(super) fn sync_native_menu_state(workspace: &DocumentWorkspace) {
    #[cfg(target_os = "macos")]
    {
        macos_menu::set_document_output_enabled(workspace.active_document().is_some());
        macos_menu::set_outline_available(
            workspace
                .active_document()
                .is_some_and(|document| document.mode() == crate::document::DocumentMode::Readonly),
        );
    }
}

pub(super) fn sync_native_theme_state(preference: ThemePreference) {
    #[cfg(target_os = "macos")]
    macos_menu::set_selected_theme(preference);
}

pub(super) fn sync_native_language_state(language: AppLanguage) {
    #[cfg(target_os = "macos")]
    macos_menu::set_selected_language(language);
}

pub(super) fn sync_workspace_state(
    window: &Window,
    webview: &WebView,
    native_footer: &NativeFooter,
    workspace: &DocumentWorkspace,
    status: &str,
) {
    update_window_title(window, workspace.active_window_title().as_deref());
    sync_native_window_state(window, workspace);
    native_footer.sync(workspace, status);
    evaluate_workspace_script(webview, "window.updateWorkspaceState", workspace, status);
}

pub(super) fn render_error_status(webview: &WebView, message: &str) {
    render_error_status_with_persistence(webview, message, false);
}

pub(super) fn render_export_error_status(webview: &WebView, message: &str) {
    render_error_status_with_persistence(webview, message, true);
}

fn render_error_status_with_persistence(webview: &WebView, message: &str, persistent: bool) {
    let payload = ErrorStatusPayload {
        message,
        persistent,
    };
    if let Ok(serialized) = serde_json::to_string(&payload) {
        let _ = webview.evaluate_script(&format!("window.showErrorStatus({serialized});"));
    }
}

pub(super) fn render_export_success(
    webview: &WebView,
    message: &str,
    output_path: &std::path::Path,
    action_label: &str,
) {
    let payload = ExportSuccessPayload {
        message,
        output_path: output_path.display().to_string(),
        action_label,
    };
    if let Ok(serialized) = serde_json::to_string(&payload) {
        let _ = webview.evaluate_script(&format!("window.showExportSuccess({serialized});"));
    }
}

pub(super) fn render_about(webview: &WebView) {
    let script = format!(
        "window.showAbout({{version:{}, author:{}, githubUrl:{}, buildTarget:{}, buildPlatform:{}}});",
        serde_json::to_string(APP_VERSION).unwrap_or_else(|_| "\"0.7.0\"".to_string()),
        serde_json::to_string(APP_AUTHOR).unwrap_or_else(|_| "\"Ronnie Deng\"".to_string()),
        serde_json::to_string(APP_GITHUB_URL)
            .unwrap_or_else(|_| "\"https://github.com/phpple/markhola\"".to_string()),
        serde_json::to_string(APP_BUILD_TARGET).unwrap_or_else(|_| "\"unknown\"".to_string()),
        serde_json::to_string(APP_BUILD_PLATFORM).unwrap_or_else(|_| "\"unknown\"".to_string()),
    );
    let _ = webview.evaluate_script(&script);
}

fn render_workspace(webview: &WebView, workspace: &DocumentWorkspace, status: &str) {
    evaluate_workspace_script(webview, "window.renderWorkspace", workspace, status);
}

fn update_window_title(window: &Window, title: Option<&str>) {
    window.set_title(title.unwrap_or(WINDOW_TITLE));
}

fn sync_native_window_state(window: &Window, workspace: &DocumentWorkspace) {
    let dirty = workspace
        .active_document()
        .is_some_and(|document| document.is_dirty());
    native_tabs::set_document_edited(window, dirty);
    native_tabs::sync_shortcut_accessories(window);
}

fn workspace_presentation(workspace: &DocumentWorkspace, status: &str) -> WorkspacePresentation {
    WorkspacePresentation {
        tabs: workspace.tab_snapshots(),
        active_document: workspace.active_document_snapshot(),
        status_message: status.to_string(),
    }
}

fn evaluate_workspace_script(
    webview: &WebView,
    function_name: &str,
    workspace: &DocumentWorkspace,
    status: &str,
) {
    let payload = workspace_presentation(workspace, status);
    let serialized = match serde_json::to_string(&payload) {
        Ok(serialized) => serialized,
        Err(error) => {
            let message =
                text("status.failed_serialize_workspace").replace("{error}", &error.to_string());
            render_error_status(webview, &message);
            return;
        }
    };
    if let Err(error) = webview.evaluate_script(&format!("{function_name}({serialized});")) {
        let message = text("status.webview_error").replace("{error}", &error.to_string());
        render_error_status(webview, &message);
    }
}
