use std::ffi::OsString;
use std::fs;

use super::parser::{CliTheme, Command, parse};
use super::{EXIT_SOURCE, EXIT_USAGE, run_if_requested};
use crate::export_service::ExportFormat;

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[test]
fn parses_frozen_public_commands_and_theme_mapping() {
    let command = parse(&args(&[
        "export-png",
        "--source=/tmp/source.md",
        "--target=/tmp/output.png",
        "--theme=dark",
        "--overwrite",
        "--json",
    ]))
    .unwrap();
    let Command::Export(options) = command else {
        panic!("expected export command");
    };
    assert_eq!(options.command, "export-png");
    assert_eq!(options.format, ExportFormat::Png);
    assert_eq!(options.theme, CliTheme::Dark);
    assert!(options.overwrite);
    assert!(options.json);

    let Command::Export(defaults) = parse(&args(&[
        "export-html",
        "--source=/tmp/source.md",
        "--target=/tmp/output.html",
    ]))
    .unwrap()
    else {
        panic!("expected export command");
    };
    assert_eq!(defaults.theme, CliTheme::Light);
    assert!(!defaults.overwrite);
}

#[test]
fn rejects_unknown_duplicate_and_out_of_scope_arguments() {
    for invalid in [
        args(&["unknown"]),
        args(&[
            "export-pdf",
            "--source=/tmp/a.md",
            "--target=/tmp/a.pdf",
            "--theme=system",
        ]),
        args(&[
            "export-pdf",
            "--source=/tmp/a.md",
            "--source=/tmp/b.md",
            "--target=/tmp/a.pdf",
        ]),
        args(&["version", "--overwrite"]),
    ] {
        assert!(parse(&invalid).is_err());
    }
    assert_eq!(run_if_requested(&args(&["unknown"])), Some(EXIT_USAGE));
}

#[test]
fn source_validation_fails_before_export_side_effects() {
    let root = std::env::temp_dir().join(format!("markhola-cli-source-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let target = root.join("output.html");
    let code = run_if_requested(&args(&[
        "export-html",
        "--source=relative.md",
        &format!("--target={}", target.display()),
    ]));
    assert_eq!(code, Some(EXIT_SOURCE));
    assert!(!target.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn public_help_excludes_internal_smoke_entry_points() {
    let help = super::output::help_text();
    assert!(help.contains("export-png"));
    assert!(help.contains("--theme=light|dark"));
    assert!(!help.contains("--smoke-"));
}
