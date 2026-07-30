use std::ffi::OsString;
use std::path::PathBuf;

use crate::export_service::ExportFormat;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CliTheme {
    Light,
    Dark,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct ExportOptions {
    pub(super) command: &'static str,
    pub(super) format: ExportFormat,
    pub(super) source: PathBuf,
    pub(super) target: PathBuf,
    pub(super) theme: CliTheme,
    pub(super) overwrite: bool,
    pub(super) json: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Command {
    Export(ExportOptions),
    Version { json: bool },
    Help { json: bool },
}

pub(super) fn is_public_invocation(args: &[OsString]) -> bool {
    let Some(first) = args.first().and_then(|value| value.to_str()) else {
        return !args.is_empty();
    };
    matches!(
        first,
        "export-png" | "export-pdf" | "export-html" | "version" | "help"
    ) || first.starts_with('-')
        || !looks_like_markdown_path(first)
}

pub(super) fn parse(args: &[OsString]) -> Result<Command, String> {
    let command = args
        .first()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "A public command is required and must be valid UTF-8.".to_string())?;
    match command {
        "export-png" => parse_export("export-png", ExportFormat::Png, &args[1..]),
        "export-pdf" => parse_export("export-pdf", ExportFormat::Pdf, &args[1..]),
        "export-html" => parse_export("export-html", ExportFormat::Html, &args[1..]),
        "version" => parse_info_command("version", &args[1..], true),
        "help" => parse_info_command("help", &args[1..], false),
        _ => Err(format!("Unknown command: {command}")),
    }
}

fn parse_export(
    command: &'static str,
    format: ExportFormat,
    args: &[OsString],
) -> Result<Command, String> {
    let mut source = None;
    let mut target = None;
    let mut theme = CliTheme::Light;
    let mut theme_seen = false;
    let mut overwrite = false;
    let mut json = false;

    for argument in args {
        let value = argument
            .to_str()
            .ok_or_else(|| "CLI arguments must be valid UTF-8.".to_string())?;
        if let Some(path) = value.strip_prefix("--source=") {
            set_once(&mut source, PathBuf::from(path), "--source")?;
        } else if let Some(path) = value.strip_prefix("--target=") {
            set_once(&mut target, PathBuf::from(path), "--target")?;
        } else if let Some(value) = value.strip_prefix("--theme=") {
            if theme_seen {
                return Err("--theme may only be provided once.".to_string());
            }
            theme_seen = true;
            theme = match value {
                "light" => CliTheme::Light,
                "dark" => CliTheme::Dark,
                _ => return Err("--theme must be light or dark.".to_string()),
            };
        } else if value == "--overwrite" {
            if overwrite {
                return Err("--overwrite may only be provided once.".to_string());
            }
            overwrite = true;
        } else if value == "--json" {
            if json {
                return Err("--json may only be provided once.".to_string());
            }
            json = true;
        } else {
            return Err(format!("Unknown argument: {value}"));
        }
    }

    Ok(Command::Export(ExportOptions {
        command,
        format,
        source: source.ok_or_else(|| "--source is required.".to_string())?,
        target: target.ok_or_else(|| "--target is required.".to_string())?,
        theme,
        overwrite,
        json,
    }))
}

fn parse_info_command(
    command: &'static str,
    args: &[OsString],
    version: bool,
) -> Result<Command, String> {
    let json = match args {
        [] => false,
        [argument] if argument == "--json" => true,
        _ => return Err(format!("{command} only accepts the optional --json argument.")),
    };
    if version {
        Ok(Command::Version { json })
    } else {
        Ok(Command::Help { json })
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, name: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("{name} may only be provided once."));
    }
    *slot = Some(value);
    Ok(())
}

fn looks_like_markdown_path(value: &str) -> bool {
    let path = std::path::Path::new(value);
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("md" | "markdown")
    )
}
