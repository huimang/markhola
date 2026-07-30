use std::path::PathBuf;

use rfd::FileDialog;
use tao::window::Window;
use wry::WebView;

use crate::app::text;
use crate::document::ActiveDocument;
use crate::save_service;
use crate::workspace::DocumentWorkspace;

use super::asset_access::{AssetAccessRegistry, register_document};
use super::native_footer::NativeFooter;
use super::workspace_view::{render_error_status, sync_workspace_state};

pub(super) fn save_document(document: &mut ActiveDocument) -> Result<(), String> {
    save_service::save_document(document)
        .map(|_| ())
        .map_err(|failure| failure.message)
}

fn save_document_as(
    document: &mut ActiveDocument,
    path: &std::path::Path,
    overwrite: bool,
) -> Result<PathBuf, String> {
    save_service::save_document_as(document, path, overwrite).map_err(|failure| failure.message)
}

pub(super) fn save_active_document(
    window: &Window,
    webview: &WebView,
    native_footer: &NativeFooter,
    workspace: &mut DocumentWorkspace,
    asset_access: &AssetAccessRegistry,
) -> bool {
    if workspace
        .active_document()
        .map(ActiveDocument::is_draft)
        .unwrap_or(false)
    {
        return save_active_document_as(window, webview, native_footer, workspace, asset_access);
    }

    let Some(document) = workspace.active_document_mut() else {
        render_error_status(webview, text("status.no_document_to_save"));
        return false;
    };
    if let Err(message) = save_document(document) {
        render_error_status(webview, &message);
        return false;
    }
    sync_workspace_state(
        window,
        webview,
        native_footer,
        workspace,
        text("status.saved"),
    );
    true
}

pub(super) fn save_active_document_as(
    window: &Window,
    webview: &WebView,
    native_footer: &NativeFooter,
    workspace: &mut DocumentWorkspace,
    asset_access: &AssetAccessRegistry,
) -> bool {
    let Some(document) = workspace.active_document() else {
        render_error_status(webview, text("status.no_document_to_save"));
        return false;
    };

    let snapshot = SaveAsSnapshot::from_document(document);
    let Some(path) = choose_save_as_path(&snapshot) else {
        return false;
    };
    if workspace
        .find_by_path_excluding(&path, snapshot.document_id)
        .is_some()
    {
        render_error_status(webview, text("status.save_target_open"));
        return false;
    }
    let Some(document) = workspace.active_document_mut() else {
        render_error_status(webview, text("status.no_document_to_save"));
        return false;
    };
    let path = match save_document_as(document, &path, true) {
        Ok(path) => path,
        Err(failure) => {
            render_error_status(webview, &failure);
            return false;
        }
    };
    if let Err(error) = register_document(asset_access, snapshot.document_id, &path) {
        let message = text("status.failed_local_assets").replace("{error}", &error.to_string());
        render_error_status(webview, &message);
        return false;
    }
    sync_workspace_state(
        window,
        webview,
        native_footer,
        workspace,
        text("status.saved_new_path"),
    );
    true
}

struct SaveAsSnapshot {
    document_id: u64,
    directory: PathBuf,
    file_name: String,
}

impl SaveAsSnapshot {
    fn from_document(document: &ActiveDocument) -> Self {
        Self {
            document_id: document.id(),
            directory: document
                .file_path()
                .parent()
                .unwrap_or(document.file_path())
                .to_path_buf(),
            file_name: document.file_name().to_string(),
        }
    }
}

fn choose_save_as_path(snapshot: &SaveAsSnapshot) -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Markdown", &["md", "markdown"])
        .set_title(text("dialog.save_markdown_as"))
        .set_directory(&snapshot.directory)
        .set_file_name(&snapshot.file_name)
        .save_file()
}

#[cfg(test)]
#[path = "save_actions/tests.rs"]
mod tests;
