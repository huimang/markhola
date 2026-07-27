use serde_json::Value;
use tao::event_loop::EventLoopProxy;
use tao::window::WindowId;
use url::Url;

use super::{UserEvent, dispatch_user_event, log_event, new_action_context};

pub(super) fn handle_ipc_message(
    proxy: &EventLoopProxy<UserEvent>,
    window_id: WindowId,
    payload: String,
) {
    log_event(
        "ipc.received",
        None,
        "ipc payload received",
        format!("payload={payload}"),
    );
    let Ok(value) = serde_json::from_str::<Value>(&payload) else {
        log_event("ipc.error", None, "ipc payload parsing failed", "");
        return;
    };

    match value.get("kind").and_then(Value::as_str) {
        Some("debug-log") => {
            if let Some(message) = value.get("message").and_then(Value::as_str) {
                log_event("ipc.debug", None, "debug log", message);
            }
        }
        Some("open-file") => {
            let ctx = new_action_context("ipc-open-file");
            dispatch_user_event(proxy, "ipc", UserEvent::OpenFile(ctx));
        }
        Some("new-document") => dispatch_user_event(proxy, "ipc", UserEvent::NewDocument),
        Some("shell-ready") => dispatch_user_event(proxy, "ipc", UserEvent::ShellReady(window_id)),
        Some("toggle-mode") => dispatch_user_event(proxy, "ipc", UserEvent::ToggleMode),
        Some("toggle-outline") => dispatch_user_event(proxy, "ipc", UserEvent::ToggleOutline),
        Some("increase-document-size") => {
            dispatch_user_event(proxy, "ipc", UserEvent::IncreaseDocumentSize)
        }
        Some("decrease-document-size") => {
            dispatch_user_event(proxy, "ipc", UserEvent::DecreaseDocumentSize)
        }
        Some("close-current-document") => {
            dispatch_user_event(proxy, "ipc", UserEvent::CloseCurrentDocument)
        }
        Some("request-save") => dispatch_user_event(proxy, "ipc", UserEvent::SaveDocument),
        Some("request-save-as") => dispatch_user_event(proxy, "ipc", UserEvent::SaveDocumentAs),
        Some("request-export-pdf") => dispatch_user_event(proxy, "ipc", UserEvent::ExportPdf),
        Some("request-export-html") => dispatch_user_event(proxy, "ipc", UserEvent::ExportHtml),
        Some("request-print") => dispatch_user_event(proxy, "ipc", UserEvent::PrintDocument),
        Some("request-open-find") => dispatch_user_event(proxy, "ipc", UserEvent::OpenFind),
        Some("request-exit") => dispatch_user_event(proxy, "ipc", UserEvent::Exit),
        Some("open-external") => {
            dispatch_string_event(value.get("href"), proxy, UserEvent::OpenExternal)
        }
        Some("open-markdown-link") => dispatch_open_path_event(value.get("href"), proxy),
        Some("activate-document") => {
            dispatch_u64_event(value.get("documentId"), proxy, UserEvent::ActivateDocument)
        }
        Some("close-document") => {
            dispatch_u64_event(value.get("documentId"), proxy, UserEvent::CloseDocument)
        }
        Some("editor-changed") => {
            if let Some(markdown) = value.get("markdown").and_then(Value::as_str) {
                dispatch_user_event(
                    proxy,
                    "ipc",
                    UserEvent::EditorChanged(window_id, markdown.to_string()),
                );
            }
        }
        _ => {}
    }
}

fn dispatch_string_event(
    value: Option<&Value>,
    proxy: &EventLoopProxy<UserEvent>,
    build: fn(String) -> UserEvent,
) {
    if let Some(value) = value.and_then(Value::as_str) {
        dispatch_user_event(proxy, "ipc", build(value.to_string()));
    }
}

fn dispatch_u64_event(
    value: Option<&Value>,
    proxy: &EventLoopProxy<UserEvent>,
    build: fn(u64) -> UserEvent,
) {
    if let Some(value) = value.and_then(Value::as_u64) {
        dispatch_user_event(proxy, "ipc", build(value));
    }
}

fn dispatch_open_path_event(value: Option<&Value>, proxy: &EventLoopProxy<UserEvent>) {
    let Some(href) = value.and_then(Value::as_str) else {
        return;
    };
    let Some(path) = markdown_path_from_href(href) else {
        log_event(
            "ipc.error",
            None,
            "open-markdown-link ignored invalid href",
            format!("href={href}"),
        );
        return;
    };
    let ctx = new_action_context("ipc-open-markdown-link");
    dispatch_user_event(
        proxy,
        "ipc",
        UserEvent::OpenPath(super::OpenPathRequest { ctx, path }),
    );
}

pub(crate) fn markdown_path_from_href(href: &str) -> Option<std::path::PathBuf> {
    let mut url = Url::parse(href).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    if !is_markdown_path(url.path()) {
        return None;
    }
    url.set_query(None);
    url.set_fragment(None);
    url.to_file_path().ok()
}

fn is_markdown_path(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown")
}
