use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::document::{ActiveDocument, DocumentMode};
use crate::file_io;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub(crate) struct SaveError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl SaveError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub(crate) fn save_document(document: &mut ActiveDocument) -> Result<PathBuf, SaveError> {
    ensure_writable(document)?;
    if document.is_draft() {
        return Err(SaveError::new(
            "save_path_required",
            "The document does not have an existing save path.",
        ));
    }
    let target = validate_existing_source(document)?;
    let disk_markdown = file_io::load_markdown(&target).map_err(|_| {
        SaveError::new(
            "external_source_changed",
            "The source file cannot be verified before saving.",
        )
    })?;
    if disk_markdown != document.saved_markdown() {
        return Err(SaveError::new(
            "external_source_changed",
            "The source file changed outside MarkHola.",
        ));
    }
    atomic_write(&target, document.markdown().as_bytes(), true)?;
    document.mark_saved();
    Ok(target)
}

pub(crate) fn save_document_as(
    document: &mut ActiveDocument,
    requested_path: &Path,
    overwrite: bool,
) -> Result<PathBuf, SaveError> {
    ensure_writable(document)?;
    let target = validate_save_as_target(requested_path, overwrite)?;
    if !document.is_draft() && target == document.canonical_path() {
        return Err(SaveError::new(
            "save_target_is_source",
            "Use save_document to replace the current source.",
        ));
    }
    let base_url = file_io::directory_base_url(&target)
        .map_err(|message| SaveError::new("invalid_output_path", message))?;
    atomic_write(&target, document.markdown().as_bytes(), overwrite)?;
    document.replace_file_path(target.clone(), base_url);
    Ok(target)
}

pub(crate) fn validate_save_as_target(
    requested_path: &Path,
    overwrite: bool,
) -> Result<PathBuf, SaveError> {
    if !requested_path.is_absolute()
        || requested_path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(SaveError::new(
            "invalid_output_path",
            "The save target must be absolute and normalized.",
        ));
    }
    file_io::ensure_supported_markdown_extension(requested_path)
        .map_err(|message| SaveError::new("invalid_output_extension", message))?;
    if requested_path
        .ancestors()
        .any(|path| path.extension().and_then(|value| value.to_str()) == Some("app"))
    {
        return Err(SaveError::new(
            "unsafe_output_path",
            "Application bundles cannot contain saved documents.",
        ));
    }
    let parent = requested_path.parent().ok_or_else(|| {
        SaveError::new("invalid_output_path", "The save target parent is missing.")
    })?;
    let canonical_parent = parent.canonicalize().map_err(|_| {
        SaveError::new(
            "invalid_output_path",
            "The save target parent must already exist.",
        )
    })?;
    if !canonical_parent.is_dir() {
        return Err(SaveError::new(
            "invalid_output_path",
            "The save target parent must be a directory.",
        ));
    }
    let file_name = requested_path.file_name().ok_or_else(|| {
        SaveError::new(
            "invalid_output_path",
            "The save target filename is missing.",
        )
    })?;
    let target = canonical_parent.join(file_name);
    if let Ok(metadata) = fs::symlink_metadata(&target) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(SaveError::new(
                "unsafe_output_path",
                "The save target must be a regular file.",
            ));
        }
        if !overwrite {
            return Err(SaveError::new(
                "output_exists",
                "The save target exists and overwrite is false.",
            ));
        }
    }
    Ok(target)
}

fn ensure_writable(document: &ActiveDocument) -> Result<(), SaveError> {
    if document.mode() != DocumentMode::Writable {
        return Err(SaveError::new(
            "document_readonly",
            "The document must be writable before saving.",
        ));
    }
    Ok(())
}

fn validate_existing_source(document: &ActiveDocument) -> Result<PathBuf, SaveError> {
    let path = document.file_path();
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        SaveError::new(
            "source_unavailable",
            "The existing source file is unavailable.",
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(SaveError::new(
            "unsafe_source_path",
            "The existing source must be a regular file.",
        ));
    }
    let canonical = path.canonicalize().map_err(|_| {
        SaveError::new(
            "source_unavailable",
            "The existing source cannot be canonicalized.",
        )
    })?;
    if canonical != document.canonical_path() {
        return Err(SaveError::new(
            "external_source_changed",
            "The source path identity changed outside MarkHola.",
        ));
    }
    Ok(canonical)
}

fn atomic_write(target: &Path, bytes: &[u8], overwrite: bool) -> Result<(), SaveError> {
    let parent = target.parent().expect("validated save target has a parent");
    let temporary = parent.join(format!(
        ".{}.markhola-save-{}-{}.tmp",
        target
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("document"),
        std::process::id(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed),
    ));
    let result = write_and_commit(&temporary, target, bytes, overwrite);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_and_commit(
    temporary: &Path,
    target: &Path,
    bytes: &[u8],
    overwrite: bool,
) -> Result<(), SaveError> {
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .map_err(|_| {
            SaveError::new(
                "output_unavailable",
                "The temporary save file cannot be created.",
            )
        })?;
    output
        .write_all(bytes)
        .and_then(|_| output.sync_all())
        .map_err(|_| SaveError::new("output_write_failed", "The document could not be synced."))?;

    if overwrite {
        if let Ok(metadata) = fs::symlink_metadata(target) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(SaveError::new(
                    "output_path_changed",
                    "The save target changed before commit.",
                ));
            }
        }
        fs::rename(temporary, target)
    } else {
        fs::hard_link(temporary, target).and_then(|_| fs::remove_file(temporary))
    }
    .map_err(|_| {
        SaveError::new(
            "output_commit_failed",
            "The document could not be committed atomically.",
        )
    })
}

#[cfg(test)]
#[path = "save_service/tests.rs"]
mod tests;
