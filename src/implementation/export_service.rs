use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lopdf::Document as PdfDocument;
use sha2::{Digest, Sha256};

use crate::app::AppTheme;
use crate::document::ActiveDocument;
use crate::pdf_export::RenderContext;

const MAX_WORKING_BYTES: u64 = 512 * 1024 * 1024;
const EXPORT_TIMEOUT: Duration = Duration::from_secs(60);

#[path = "export_service/cancellation.rs"]
mod cancellation;
#[path = "export_service/output.rs"]
mod output;
#[cfg(test)]
#[path = "export_service/theme_tests.rs"]
mod theme_tests;

pub use cancellation::ExportCancellation;
use cancellation::run_with_cancellation;
pub(crate) use cancellation::{
    CancelOutcome, ExportStatus, begin_export_cancellation, cancel_export_and_wait,
    cooperative_cancellation_requested, export_status, finish_export, finish_unresolved_export,
    register_queued_export,
};
#[cfg(test)]
pub(crate) use cancellation::{finish_export_cancellation, request_export_cancellation};
use output::{atomic_commit, validate_format, validate_target};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportFormat {
    Png,
    Pdf,
    Html,
}

impl ExportFormat {
    pub(super) fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Pdf => "pdf",
            Self::Html => "html",
        }
    }
}

#[derive(Debug)]
pub struct ExportError {
    pub code: &'static str,
    pub message: String,
}

impl ExportError {
    pub(super) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug)]
pub struct ExportResult {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub page_count: Option<usize>,
}

pub fn export_document_to_path(
    document: &ActiveDocument,
    format: ExportFormat,
    requested_path: &Path,
    overwrite: bool,
    cancellation: &ExportCancellation,
) -> Result<ExportResult, ExportError> {
    export_document_to_path_with_theme(
        document,
        AppTheme::Default,
        format,
        requested_path,
        overwrite,
        cancellation,
    )
}

pub fn export_document_to_path_with_theme(
    document: &ActiveDocument,
    theme: AppTheme,
    format: ExportFormat,
    requested_path: &Path,
    overwrite: bool,
    cancellation: &ExportCancellation,
) -> Result<ExportResult, ExportError> {
    export_document_to_path_with_theme_and_context(
        document,
        theme,
        RenderContext::default(),
        format,
        requested_path,
        overwrite,
        cancellation,
    )
}

pub fn export_document_to_path_with_theme_and_context(
    document: &ActiveDocument,
    theme: AppTheme,
    context: RenderContext,
    format: ExportFormat,
    requested_path: &Path,
    overwrite: bool,
    cancellation: &ExportCancellation,
) -> Result<ExportResult, ExportError> {
    let started_at = Instant::now();
    cancellation.check()?;
    let target = validate_target(requested_path, format, overwrite)?;
    let rendered = run_with_cancellation(cancellation, || render(document, theme, context, format))
        .map_err(map_render_error)?;
    if started_at.elapsed() > EXPORT_TIMEOUT {
        return Err(ExportError::new(
            "render_timeout",
            "The export exceeded the 60 second limit.",
        ));
    }
    cancellation.check()?;
    if rendered.bytes.len() as u64 > MAX_WORKING_BYTES {
        return Err(ExportError::new(
            "render_resource_limit",
            "The export output exceeds the working memory limit.",
        ));
    }
    validate_format(format, &rendered.bytes)?;
    atomic_commit(&target, &rendered.bytes, overwrite, cancellation)?;

    Ok(ExportResult {
        path: target,
        sha256: format!("{:x}", Sha256::digest(&rendered.bytes)),
        bytes: rendered.bytes.len() as u64,
        width: rendered.width,
        height: rendered.height,
        page_count: rendered.page_count,
    })
}

struct RenderedExport {
    bytes: Vec<u8>,
    width: Option<u64>,
    height: Option<u64>,
    page_count: Option<usize>,
}

fn render(
    document: &ActiveDocument,
    theme: AppTheme,
    context: RenderContext,
    format: ExportFormat,
) -> Result<RenderedExport, String> {
    match format {
        ExportFormat::Png => {
            let png = crate::pdf_export::render_document_png_data_with_theme_and_context(
                document,
                theme.key(),
                context,
            )?;
            Ok(RenderedExport {
                bytes: png.bytes,
                width: Some(png.width),
                height: Some(png.height),
                page_count: None,
            })
        }
        ExportFormat::Pdf => {
            let bytes = crate::pdf_export::render_document_pdf_data_with_theme_and_context(
                document,
                theme.key(),
                context,
            )?;
            let page_count = PdfDocument::load_mem(&bytes)
                .map_err(|error| format!("invalid_output: {error}"))?
                .get_pages()
                .len();
            Ok(RenderedExport {
                bytes,
                width: None,
                height: None,
                page_count: Some(page_count),
            })
        }
        ExportFormat::Html => {
            crate::pdf_export::validate_export_local_images(document)?;
            Ok(RenderedExport {
                bytes: crate::html_export::build_export_html_with_theme_and_context(
                    document,
                    theme.key(),
                    context,
                )
                .into_bytes(),
                width: None,
                height: None,
                page_count: None,
            })
        }
    }
}

fn map_render_error(message: String) -> ExportError {
    for code in [
        "cancelled",
        "render_timeout",
        "render_resource_limit",
        "missing_local_asset",
        "render_not_ready",
    ] {
        if message.contains(code) {
            return ExportError::new(code, message);
        }
    }
    if message.contains("Timed out") {
        ExportError::new("render_timeout", message)
    } else {
        ExportError::new("render_failed", message)
    }
}

#[cfg(test)]
#[path = "export_service/tests.rs"]
mod tests;
