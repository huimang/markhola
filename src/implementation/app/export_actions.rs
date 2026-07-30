use std::path::{Path, PathBuf};

use rfd::FileDialog;

use crate::app::text;
use crate::document::ActiveDocument;
use crate::export_service::{self, ExportCancellation, ExportFormat};
use crate::html_export::{self, HtmlExportOutcome};
use crate::pdf_export::{self, PdfExportOutcome};
use crate::printing::{self, PrintOutcome};
use crate::workspace::DocumentWorkspace;
use wry::WebView;

use super::workspace_view::{render_error_status, render_export_success};

pub(super) fn export_png(webview: &WebView, workspace: &DocumentWorkspace) {
    let Some(document) = workspace.active_document() else {
        render_error_status(webview, text("status.no_document"));
        return;
    };
    let Some(path) = choose_png_export_path(document) else {
        return;
    };
    match export_service::export_document_to_path(
        document,
        ExportFormat::Png,
        &path,
        true,
        &ExportCancellation::default(),
    ) {
        Ok(result) => render_export_success(
            webview,
            &text("status.exported_png").replace("{path}", &result.path.display().to_string()),
            &result.path,
            text("menu.open"),
        ),
        Err(failure) => {
            let message = text("status.export_png_failed").replace("{error}", &failure.message);
            render_error_status(webview, &message);
        }
    }
}

fn choose_png_export_path(document: &ActiveDocument) -> Option<PathBuf> {
    let suggested_name = suggested_png_path(document.file_path())
        .file_name()?
        .to_string_lossy()
        .into_owned();
    FileDialog::new()
        .add_filter("PNG", &["png"])
        .set_title(text("dialog.export_png"))
        .set_directory(
            document
                .file_path()
                .parent()
                .unwrap_or(document.file_path()),
        )
        .set_file_name(suggested_name)
        .save_file()
}

fn suggested_png_path(path: &Path) -> PathBuf {
    let mut output = path.to_path_buf();
    output.set_extension("png");
    output
}

pub(super) fn export_pdf(webview: &WebView, workspace: &DocumentWorkspace) {
    match workspace.active_document() {
        Some(document) => match pdf_export::export_document(document) {
            Ok(PdfExportOutcome::Exported(path)) => render_export_success(
                webview,
                &text("status.exported_pdf").replace("{path}", &path.display().to_string()),
                &path,
                text("menu.open"),
            ),
            Ok(PdfExportOutcome::Cancelled) => {}
            Err(message) => render_error_status(webview, &message),
        },
        None => render_error_status(webview, text("status.no_document")),
    }
}

pub(super) fn export_html(webview: &WebView, workspace: &DocumentWorkspace) {
    match workspace.active_document() {
        Some(document) => match html_export::export_document(document) {
            Ok(HtmlExportOutcome::Exported(path)) => render_export_success(
                webview,
                &text("status.exported_html").replace("{path}", &path.display().to_string()),
                &path,
                text("menu.open"),
            ),
            Ok(HtmlExportOutcome::Cancelled) => {}
            Err(message) => render_error_status(webview, &message),
        },
        None => render_error_status(webview, text("status.no_document")),
    }
}

pub(super) fn print_document(webview: &WebView, workspace: &DocumentWorkspace) {
    match workspace.active_document() {
        Some(document) => match printing::print_document(document) {
            Ok(PrintOutcome::Started) => {}
            Err(message) => render_error_status(webview, &message),
        },
        None => render_error_status(webview, text("status.no_document")),
    }
}

pub(super) fn open_find_panel(webview: &WebView, workspace: &DocumentWorkspace) {
    if workspace.active_document().is_some() {
        let _ = webview.evaluate_script("window.openFindPanel();");
    } else {
        render_error_status(webview, text("status.no_document"));
    }
}

#[cfg(test)]
#[path = "export_actions/tests.rs"]
mod tests;
