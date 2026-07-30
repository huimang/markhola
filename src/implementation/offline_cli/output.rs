use std::path::Path;

use serde_json::{Value, json};

use crate::export_service::{ExportFormat, ExportResult};

pub(super) const SCHEMA_VERSION: u32 = 1;

pub(super) fn print_success(
    command: &str,
    format: ExportFormat,
    source: &Path,
    result: &ExportResult,
    json_output: bool,
) {
    if json_output {
        print_json(json!({
            "schema_version": SCHEMA_VERSION,
            "command": command,
            "success": true,
            "source": source,
            "target": result.path,
            "format": format_name(format),
            "sha256": result.sha256,
            "size": result.bytes,
        }));
    } else {
        println!(
            "Exported {} to {} ({} bytes, SHA-256 {}).",
            source.display(),
            result.path.display(),
            result.bytes,
            result.sha256
        );
    }
}

pub(super) fn print_failure(
    command: &str,
    error_code: &str,
    message: &str,
    source: Option<&Path>,
    target: Option<&Path>,
    json_output: bool,
) {
    if json_output {
        let mut response = json!({
            "schema_version": SCHEMA_VERSION,
            "command": command,
            "success": false,
            "error_code": error_code,
            "message": message,
        });
        if let Some(source) = source {
            response["source"] = json!(source);
        }
        if let Some(target) = target {
            response["target"] = json!(target);
        }
        print_json(response);
    }
    eprintln!("{command} failed [{error_code}]: {message}");
}

pub(super) fn print_version(json_output: bool) {
    if json_output {
        print_json(json!({
            "schema_version": SCHEMA_VERSION,
            "command": "version",
            "success": true,
            "version": env!("CARGO_PKG_VERSION"),
        }));
    } else {
        println!("MarkHola {}", env!("CARGO_PKG_VERSION"));
    }
}

pub(super) fn print_help(json_output: bool) {
    let help = help_text();
    if json_output {
        print_json(json!({
            "schema_version": SCHEMA_VERSION,
            "command": "help",
            "success": true,
            "help": help,
        }));
    } else {
        print!("{help}");
    }
}

pub(super) fn help_text() -> &'static str {
    "MarkHola offline export\n\
\n\
Usage:\n\
  markhola export-png --source=/absolute/input.md --target=/absolute/output.png [--theme=light|dark] [--overwrite] [--json]\n\
  markhola export-pdf --source=/absolute/input.md --target=/absolute/output.pdf [--theme=light|dark] [--overwrite] [--json]\n\
  markhola export-html --source=/absolute/input.md --target=/absolute/output.html [--theme=light|dark] [--overwrite] [--json]\n\
  markhola version [--json]\n\
  markhola help [--json]\n\
\n\
The default theme is light. Existing targets are preserved unless --overwrite is provided.\n"
}

fn print_json(value: Value) {
    println!(
        "{}",
        serde_json::to_string(&value).expect("CLI JSON response is serializable")
    );
}

fn format_name(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Png => "png",
        ExportFormat::Pdf => "pdf",
        ExportFormat::Html => "html",
    }
}
