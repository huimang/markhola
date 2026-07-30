use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::app::AppTheme;
use crate::document::ActiveDocument;
use crate::export_service::{ExportCancellation, ExportError};

mod output;
mod parser;
mod runtime;

use parser::{CliTheme, Command, ExportOptions};

const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_SOURCE: i32 = 3;
const EXIT_TARGET: i32 = 4;
const EXIT_RENDER: i32 = 5;
const EXIT_RESOURCE: i32 = 6;
const EXIT_INTERNAL: i32 = 7;

pub fn run_if_requested(args: &[OsString]) -> Option<i32> {
    if !parser::is_public_invocation(args) {
        return None;
    }
    Some(match parser::parse(args) {
        Ok(Command::Version { json }) => {
            output::print_version(json);
            EXIT_SUCCESS
        }
        Ok(Command::Help { json }) => {
            output::print_help(json);
            EXIT_SUCCESS
        }
        Ok(Command::Export(options)) => run_export(options),
        Err(message) => {
            let json = args.iter().any(|argument| argument == "--json");
            let command = args
                .first()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown");
            output::print_failure(command, "cli_usage", &message, None, None, json);
            EXIT_USAGE
        }
    })
}

fn run_export(options: ExportOptions) -> i32 {
    let source = match validate_source(&options.source) {
        Ok(source) => source,
        Err(message) => {
            output::print_failure(
                options.command,
                "invalid_source",
                &message,
                None,
                None,
                options.json,
            );
            return EXIT_SOURCE;
        }
    };
    let markdown = match crate::file_io::load_markdown(&source) {
        Ok(markdown) => markdown,
        Err(message) => {
            output::print_failure(
                options.command,
                "source_unreadable",
                &message,
                Some(&source),
                None,
                options.json,
            );
            return EXIT_SOURCE;
        }
    };
    let base_url = match crate::file_io::directory_base_url(&source) {
        Ok(base_url) => base_url,
        Err(message) => {
            output::print_failure(
                options.command,
                "source_unavailable",
                &message,
                Some(&source),
                None,
                options.json,
            );
            return EXIT_SOURCE;
        }
    };
    let document = ActiveDocument::open_with_id(1, source.clone(), markdown, base_url);
    let theme = match options.theme {
        CliTheme::Light => AppTheme::Default,
        CliTheme::Dark => AppTheme::Dark,
    };
    let hidden_runtime = if matches!(
        options.format,
        crate::export_service::ExportFormat::Png | crate::export_service::ExportFormat::Pdf
    ) {
        match runtime::HiddenExportRuntime::initialize() {
            Ok(runtime) => Some(runtime),
            Err(message) => {
                output::print_failure(
                    options.command,
                    "runtime_initialization_failed",
                    &message,
                    Some(&source),
                    canonical_target_for_error(&options.target).as_deref(),
                    options.json,
                );
                return EXIT_INTERNAL;
            }
        }
    } else {
        None
    };
    let export_result = objc2::rc::autoreleasepool(|_| {
        crate::export_service::export_document_to_path_with_theme(
            &document,
            theme,
            options.format,
            &options.target,
            options.overwrite,
            &ExportCancellation::default(),
        )
    });
    if let Some(runtime) = hidden_runtime
        && let Err(message) = runtime.finish()
    {
        output::print_failure(
            options.command,
            "runtime_cleanup_failed",
            &message,
            Some(&source),
            canonical_target_for_error(&options.target).as_deref(),
            options.json,
        );
        return EXIT_INTERNAL;
    }
    match export_result {
        Ok(result) => {
            output::print_success(
                options.command,
                options.format,
                &source,
                &result,
                options.json,
            );
            EXIT_SUCCESS
        }
        Err(error) => {
            let (exit_code, target) = classify_export_error(&error, &options.target);
            output::print_failure(
                options.command,
                error.code,
                &error.message,
                Some(&source),
                target.as_deref(),
                options.json,
            );
            exit_code
        }
    }
}

fn validate_source(requested: &Path) -> Result<PathBuf, String> {
    if !requested.is_absolute()
        || requested
            .components()
            .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
    {
        return Err("The source path must be absolute and normalized.".to_string());
    }
    let metadata = fs::symlink_metadata(requested)
        .map_err(|_| "The source file does not exist or cannot be inspected.".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("The source must be a regular file and not a symlink.".to_string());
    }
    crate::file_io::ensure_supported_markdown_extension(requested)?;
    let canonical = requested
        .canonicalize()
        .map_err(|_| "The source path cannot be canonicalized.".to_string())?;
    if canonical != requested {
        return Err("The source path must already be canonical.".to_string());
    }
    if canonical
        .ancestors()
        .any(|path| path.extension().and_then(|value| value.to_str()) == Some("app"))
    {
        return Err("Application bundles cannot contain CLI source files.".to_string());
    }
    Ok(canonical)
}

fn classify_export_error(error: &ExportError, requested_target: &Path) -> (i32, Option<PathBuf>) {
    let target = canonical_target_for_error(requested_target);
    let exit_code = match error.code {
        "render_timeout" | "render_resource_limit" => EXIT_RESOURCE,
        "invalid_output_path"
        | "invalid_output_extension"
        | "unsafe_output_path"
        | "output_exists"
        | "output_unavailable"
        | "output_write_failed"
        | "output_path_changed"
        | "output_commit_failed" => EXIT_TARGET,
        "render_failed"
        | "render_not_ready"
        | "missing_local_asset"
        | "invalid_output"
        | "cancelled" => EXIT_RENDER,
        _ => EXIT_INTERNAL,
    };
    (exit_code, target)
}

fn canonical_target_for_error(target: &Path) -> Option<PathBuf> {
    let parent = target.parent()?.canonicalize().ok()?;
    Some(parent.join(target.file_name()?))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
