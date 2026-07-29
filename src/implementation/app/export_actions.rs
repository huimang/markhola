use crate::app::text;
use crate::html_export::{self, HtmlExportOutcome};
use crate::pdf_export::{self, PdfExportOutcome};
use crate::printing::{self, PrintOutcome};
use crate::workspace::DocumentWorkspace;
use wry::WebView;

use super::workspace_view::{render_error_status, render_export_success};

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
