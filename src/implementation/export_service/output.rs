use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{ExportCancellation, ExportError, ExportFormat};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

pub(super) fn validate_target(
    requested_path: &Path,
    format: ExportFormat,
    overwrite: bool,
) -> Result<PathBuf, ExportError> {
    if !requested_path.is_absolute()
        || requested_path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(ExportError::new(
            "invalid_output_path",
            "The output path must be absolute and normalized.",
        ));
    }
    if requested_path.extension().and_then(|value| value.to_str()) != Some(format.extension()) {
        return Err(ExportError::new(
            "invalid_output_extension",
            "The output extension does not match the export format.",
        ));
    }
    if requested_path
        .ancestors()
        .any(|path| path.extension().and_then(|value| value.to_str()) == Some("app"))
    {
        return Err(ExportError::new(
            "unsafe_output_path",
            "Application bundles cannot contain export output.",
        ));
    }
    let parent = requested_path
        .parent()
        .ok_or_else(|| ExportError::new("invalid_output_path", "The output parent is missing."))?;
    let canonical_parent = parent.canonicalize().map_err(|_| {
        ExportError::new(
            "invalid_output_path",
            "The output parent must already exist.",
        )
    })?;
    if !canonical_parent.is_dir() {
        return Err(ExportError::new(
            "invalid_output_path",
            "The output parent must be a directory.",
        ));
    }
    let file_name = requested_path.file_name().ok_or_else(|| {
        ExportError::new("invalid_output_path", "The output filename is missing.")
    })?;
    let target = canonical_parent.join(file_name);
    if let Ok(metadata) = fs::symlink_metadata(&target) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(ExportError::new(
                "unsafe_output_path",
                "The output target must be a regular file.",
            ));
        }
        if !overwrite {
            return Err(ExportError::new(
                "output_exists",
                "The output already exists and overwrite is false.",
            ));
        }
    }
    Ok(target)
}

pub(super) fn atomic_commit(
    target: &Path,
    bytes: &[u8],
    overwrite: bool,
    cancellation: &ExportCancellation,
) -> Result<(), ExportError> {
    let parent = target.parent().expect("validated target has a parent");
    let temporary = parent.join(format!(
        ".{}.markhola-export-{}-{}.tmp",
        target
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("output"),
        std::process::id(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed),
    ));
    let result = write_and_commit(&temporary, target, bytes, overwrite, cancellation);
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
    cancellation: &ExportCancellation,
) -> Result<(), ExportError> {
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .map_err(|_| {
            ExportError::new(
                "output_unavailable",
                "The temporary output cannot be created.",
            )
        })?;
    output.write_all(bytes).map_err(|_| {
        ExportError::new(
            "output_write_failed",
            "The export output could not be written.",
        )
    })?;
    output.sync_all().map_err(|_| {
        ExportError::new(
            "output_write_failed",
            "The export output could not be synced.",
        )
    })?;
    cancellation.check()?;
    if let Ok(metadata) = fs::symlink_metadata(target)
        && (metadata.file_type().is_symlink() || !metadata.is_file() || !overwrite)
    {
        return Err(ExportError::new(
            "output_path_changed",
            "The output target changed before commit.",
        ));
    }
    fs::rename(temporary, target).map_err(|_| {
        ExportError::new(
            "output_commit_failed",
            "The export output could not be committed.",
        )
    })
}

pub(super) fn validate_format(format: ExportFormat, bytes: &[u8]) -> Result<(), ExportError> {
    let valid = match format {
        ExportFormat::Png => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        ExportFormat::Pdf => bytes.starts_with(b"%PDF-"),
        ExportFormat::Html => {
            let text = std::str::from_utf8(bytes).unwrap_or_default();
            text.contains("<!DOCTYPE html>") && text.contains("</html>")
        }
    };
    if valid {
        Ok(())
    } else {
        Err(ExportError::new(
            "invalid_output",
            "The renderer returned an invalid output format.",
        ))
    }
}
