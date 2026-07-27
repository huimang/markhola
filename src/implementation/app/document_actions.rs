use std::path::PathBuf;
use std::time::SystemTime;

use rfd::FileDialog;

use crate::app::text;
use crate::document::ActiveDocument;
use crate::file_io;
use crate::workspace::{DocumentWorkspace, WorkspaceOpenResult};

use super::asset_access::{AssetAccessRegistry, register_document};
use super::log_event;

pub(super) fn open_documents_dialog(event_id: u64) -> Option<Vec<PathBuf>> {
    let started_at = SystemTime::now();
    log_event(
        "file_dialog.begin",
        Some(event_id),
        "opening file dialog",
        "",
    );
    let result = FileDialog::new()
        .add_filter("Markdown", &["md", "markdown"])
        .set_title(text("dialog.open_markdown"))
        .pick_files();
    let elapsed_ms = started_at
        .elapsed()
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    log_event(
        "file_dialog.end",
        Some(event_id),
        "file dialog finished",
        format!("selected={} elapsed_ms={elapsed_ms}", result.is_some()),
    );
    result
}

pub(super) fn create_blank_document_model(workspace: &mut DocumentWorkspace) -> u64 {
    let document_id = workspace.next_document_id();
    let blank_number = workspace.next_blank_document_number();
    let document = ActiveDocument::new_blank_with_id(document_id, blank_number);
    workspace.open_document(document);
    document_id
}

pub(super) fn open_document_model(
    workspace: &mut DocumentWorkspace,
    path: &PathBuf,
    event_id: Option<u64>,
    asset_access: &AssetAccessRegistry,
) -> Result<WorkspaceOpenResult, String> {
    log_event(
        "open_document.begin",
        event_id,
        "open_document start",
        format!("path={}", path.display()),
    );
    if let Some(document_id) = workspace.find_by_path(path) {
        workspace.activate_document(document_id);
        return Ok(WorkspaceOpenResult::ActivatedExisting(document_id));
    }

    let document = match load_document(workspace.next_document_id(), path) {
        Ok(document) => document,
        Err(message) => {
            log_event(
                "open_document.end",
                event_id,
                "open_document failed",
                format!("path={} error={message}", path.display()),
            );
            return Err(message);
        }
    };
    log_event(
        "open_document.end",
        event_id,
        "open_document success",
        format!("path={}", path.display()),
    );
    let result = workspace.open_document(document);
    if let WorkspaceOpenResult::OpenedNew(document_id) = result {
        if let Err(error) = register_document(asset_access, document_id, path) {
            workspace.close_document(document_id);
            return Err(text("status.failed_local_assets").replace("{error}", &error.to_string()));
        }
    }
    Ok(result)
}

pub(crate) fn load_document(document_id: u64, path: &PathBuf) -> Result<ActiveDocument, String> {
    log_event(
        "load_document.begin",
        None,
        "load_document path",
        format!("path={}", path.display()),
    );
    let markdown = file_io::load_markdown(path)?;
    let base_url = file_io::directory_base_url(path)?;
    Ok(ActiveDocument::open_with_id(
        document_id,
        path.clone(),
        markdown,
        base_url,
    ))
}

pub(crate) fn reload_workspace_documents_from_disk(
    workspace: &mut DocumentWorkspace,
) -> Result<String, String> {
    let document_ids = workspace.document_ids();
    let mut reloaded = 0usize;
    let mut skipped_dirty = 0usize;
    let mut failures = Vec::new();

    for document_id in document_ids {
        let Some(document) = workspace.document_by_id_mut(document_id) else {
            continue;
        };
        if document.is_dirty() {
            skipped_dirty += 1;
            continue;
        }
        let path = document.file_path().to_path_buf();
        match file_io::load_markdown(&path) {
            Ok(markdown) => {
                document.reload_from_disk_markdown(markdown);
                reloaded += 1;
            }
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    failures.first().map_or_else(
        || Ok(reload_status_message(reloaded, skipped_dirty)),
        |failure| Err(format!("Reload failed: {failure}")),
    )
}

fn reload_status_message(reloaded: usize, skipped_dirty: usize) -> String {
    match (reloaded, skipped_dirty) {
        (0, 0) | (_, 0) => text("status.reloaded").to_string(),
        (_, 1) => text("status.reloaded_one_dirty").to_string(),
        (_, count) => text("status.reloaded_dirty").replace("{count}", &count.to_string()),
    }
}
