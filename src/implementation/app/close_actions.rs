use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tao::window::Window;
use wry::WebView;

use crate::app::text;
use crate::document::ActiveDocument;
use crate::workspace::DocumentWorkspace;

use super::PendingChangesAction;
use super::asset_access::{AssetAccessRegistry, unregister_document};
use super::save_actions::save_document;
use super::workspace_view::render_status;

pub(super) fn resolve_all_pending_changes(
    window: &Window,
    webview: &WebView,
    workspace: &mut DocumentWorkspace,
) -> bool {
    let document_ids = workspace
        .tab_snapshots()
        .into_iter()
        .map(|tab| tab.document_id)
        .collect::<Vec<_>>();
    for document_id in document_ids {
        let Some(document) = workspace.document_by_id_mut(document_id) else {
            continue;
        };
        if !resolve_document_pending_changes(window, webview, document) {
            return false;
        }
    }
    true
}

pub(super) fn close_document_model(
    window: &Window,
    webview: &WebView,
    workspace: &mut DocumentWorkspace,
    document_id: u64,
    asset_access: &AssetAccessRegistry,
) -> bool {
    let Some(document) = workspace.document_by_id_mut(document_id) else {
        render_status(webview, text("status.missing_tab"), "error");
        return false;
    };
    if !resolve_document_pending_changes(window, webview, document) {
        return false;
    }
    workspace.close_document(document_id);
    unregister_document(asset_access, document_id);
    true
}

pub(super) fn close_document_models(
    window: &Window,
    webview: &WebView,
    workspace: &mut DocumentWorkspace,
    document_ids: &[u64],
    asset_access: &AssetAccessRegistry,
) -> bool {
    for document_id in document_ids {
        let Some(document) = workspace.document_by_id_mut(*document_id) else {
            continue;
        };
        if !resolve_document_pending_changes(window, webview, document) {
            return false;
        }
    }
    for document_id in document_ids {
        workspace.close_document(*document_id);
        unregister_document(asset_access, *document_id);
    }
    true
}

pub(super) fn resolve_document_pending_changes(
    window: &Window,
    webview: &WebView,
    document: &mut ActiveDocument,
) -> bool {
    if !document.is_dirty() {
        return true;
    }
    match ask_pending_changes_action(window, document.file_name()) {
        PendingChangesAction::Save => match save_document(document) {
            Ok(()) => true,
            Err(message) => {
                render_status(webview, &message, "error");
                false
            }
        },
        PendingChangesAction::Discard => true,
        PendingChangesAction::Cancel => {
            render_status(webview, text("status.action_cancelled"), "info");
            false
        }
    }
}

fn ask_pending_changes_action(window: &Window, file_name: &str) -> PendingChangesAction {
    let result = MessageDialog::new()
        .set_parent(window)
        .set_level(MessageLevel::Warning)
        .set_title(text("dialog.unsaved_title"))
        .set_description(text("dialog.unsaved_description").replace("{file_name}", file_name))
        .set_buttons(MessageButtons::YesNoCancelCustom(
            text("dialog.save").to_string(),
            text("dialog.discard").to_string(),
            text("dialog.cancel").to_string(),
        ))
        .show();

    match result {
        MessageDialogResult::Custom(choice) if choice == text("dialog.save") => {
            PendingChangesAction::Save
        }
        MessageDialogResult::Custom(choice) if choice == text("dialog.discard") => {
            PendingChangesAction::Discard
        }
        _ => PendingChangesAction::Cancel,
    }
}
