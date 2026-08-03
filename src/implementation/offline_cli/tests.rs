use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::sync::atomic::{AtomicU64, Ordering};

use super::parser::{CliTheme, Command, parse};
use super::{
    EXIT_INTERNAL, EXIT_RENDER, EXIT_RESOURCE, EXIT_SOURCE, EXIT_TARGET, EXIT_USAGE,
    classify_export_error, run_if_requested,
};
use crate::export_service::{ExportError, ExportFormat};

static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

fn temp_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "markhola-offline-cli-{label}-{}-{}",
        std::process::id(),
        NEXT_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_fixture(root: &Path) -> PathBuf {
    let source = root.join("fixture.md");
    fs::write(
        &source,
        include_str!("../../tests/fixtures/v0.9.2-offline-cli-export.md"),
    )
    .unwrap();
    fs::write(
        root.join("local-diagram.svg"),
        include_str!("../../tests/fixtures/local-diagram.svg"),
    )
    .unwrap();
    source
}

fn run_markhola(args: &[&str]) -> std::process::Output {
    ProcessCommand::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--bin")
        .arg("markhola")
        .arg("--")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("markhola CLI command should start")
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

#[test]
fn offline_exports_use_a_prohibited_unattached_runtime_without_app_bootstrap() {
    let main = include_str!("../../main.rs");
    let cli = include_str!("mod.rs");
    let runtime = include_str!("runtime.rs");

    assert!(main.find("--smoke-export").unwrap() < main.find("run_if_requested").unwrap());
    assert!(main.find("run_if_requested").unwrap() < main.find("app::run()").unwrap());
    assert!(cli.contains("HiddenExportRuntime::initialize()"));
    assert!(cli.contains("export_document_to_path_with_theme("));
    assert!(runtime.contains("NSApplicationActivationPolicy::Prohibited"));
    assert!(runtime.contains("application.windows().count() != 0"));
    assert!(!runtime.contains("activateIgnoringOtherApps"));
    assert!(!runtime.contains("NSWindow"));
    assert!(!cli.contains("ProtocolTransport"));
    assert!(!cli.contains("FileDialog"));
}

#[test]
fn symlink_source_and_existing_target_fail_closed() {
    use std::os::unix::fs::symlink;

    let root = std::env::temp_dir().join(format!("markhola-cli-paths-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let source = root.join("source.md");
    let source_link = root.join("source-link.md");
    let target = root.join("output.html");
    fs::write(&source, "# Source remains unchanged").unwrap();
    symlink(&source, &source_link).unwrap();
    fs::write(&target, "existing").unwrap();
    let before = fs::read(&source).unwrap();

    assert_eq!(
        run_if_requested(&args(&[
            "export-html",
            &format!("--source={}", source_link.display()),
            &format!("--target={}", root.join("linked.html").display()),
        ])),
        Some(EXIT_SOURCE)
    );
    assert_eq!(
        run_if_requested(&args(&[
            "export-html",
            &format!("--source={}", source.display()),
            &format!("--target={}", target.display()),
        ])),
        Some(EXIT_TARGET)
    );
    assert_eq!(fs::read(&source).unwrap(), before);
    assert_eq!(fs::read_to_string(&target).unwrap(), "existing");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn export_errors_map_to_the_frozen_exit_code_classes() {
    for (code, expected) in [
        ("invalid_output_path", EXIT_TARGET),
        ("output_exists", EXIT_TARGET),
        ("render_failed", EXIT_RENDER),
        ("missing_local_asset", EXIT_RENDER),
        ("render_timeout", EXIT_RESOURCE),
        ("render_resource_limit", EXIT_RESOURCE),
        ("unexpected", EXIT_INTERNAL),
    ] {
        let error = ExportError {
            code,
            message: "fixture".to_string(),
        };
        assert_eq!(
            classify_export_error(&error, std::path::Path::new("/tmp/output.pdf")).0,
            expected
        );
    }
}

#[test]
fn info_commands_emit_schema_v1_json_and_private_help() {
    let version = run_markhola(&["version", "--json"]);
    assert!(version.status.success(), "version stderr:\n{}", String::from_utf8_lossy(&version.stderr));
    let version_json: serde_json::Value = serde_json::from_slice(&version.stdout).unwrap();
    assert_eq!(version_json["schema_version"], 1);
    assert_eq!(version_json["command"], "version");
    assert_eq!(version_json["success"], true);
    assert!(version_json["version"].as_str().unwrap().starts_with("0.9.3"));

    let help = run_markhola(&["help", "--json"]);
    assert!(help.status.success(), "help stderr:\n{}", String::from_utf8_lossy(&help.stderr));
    let help_json: serde_json::Value = serde_json::from_slice(&help.stdout).unwrap();
    assert_eq!(help_json["schema_version"], 1);
    assert_eq!(help_json["command"], "help");
    assert_eq!(help_json["success"], true);
    let help_text = help_json["help"].as_str().unwrap();
    assert!(help_text.contains("export-png"));
    assert!(help_text.contains("--theme=light|dark"));
    assert!(!help_text.contains("--smoke-"));
}

#[test]
fn successful_html_cli_export_preserves_source_and_reports_canonical_json() {
    let root = temp_root("html-success");
    let source = write_fixture(&root).canonicalize().unwrap();
    let target = root.join("result.html");
    let before = fs::read_to_string(&source).unwrap();

    let output = run_markhola(&[
        "export-html",
        &format!("--source={}", source.display()),
        &format!("--target={}", target.display()),
        "--json",
    ]);

    assert_eq!(output.status.code(), Some(0), "stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "export-html");
    assert_eq!(json["success"], true);
    assert_eq!(json["format"], "html");
    assert_eq!(json["source"], source.to_string_lossy().as_ref());
    assert_eq!(
        json["target"],
        target.canonicalize().unwrap().to_string_lossy().as_ref()
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), before);
    assert!(fs::read_to_string(&target).unwrap().contains("Offline CLI Export Fixture"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cli_exports_real_png_pdf_and_html_with_light_dark_parity() {
    let root = temp_root("theme-parity");
    let source = write_fixture(&root).canonicalize().unwrap();

    for (command, extension) in [
        ("export-html", "html"),
        ("export-pdf", "pdf"),
        ("export-png", "png"),
    ] {
        let light = root.join(format!("{command}-light.{extension}"));
        let dark = root.join(format!("{command}-dark.{extension}"));

        let light_output = run_markhola(&[
            command,
            &format!("--source={}", source.display()),
            &format!("--target={}", light.display()),
            "--theme=light",
            "--json",
        ]);
        assert_eq!(
            light_output.status.code(),
            Some(0),
            "{command} light stderr:\n{}",
            String::from_utf8_lossy(&light_output.stderr)
        );
        let dark_output = run_markhola(&[
            command,
            &format!("--source={}", source.display()),
            &format!("--target={}", dark.display()),
            "--theme=dark",
            "--json",
        ]);
        assert_eq!(
            dark_output.status.code(),
            Some(0),
            "{command} dark stderr:\n{}",
            String::from_utf8_lossy(&dark_output.stderr)
        );

        let light_json: serde_json::Value = serde_json::from_slice(&light_output.stdout).unwrap();
        let dark_json: serde_json::Value = serde_json::from_slice(&dark_output.stdout).unwrap();
        assert_eq!(light_json["schema_version"], 1);
        assert_eq!(dark_json["schema_version"], 1);
        assert_eq!(light_json["success"], true);
        assert_eq!(dark_json["success"], true);
        assert_ne!(light_json["sha256"], dark_json["sha256"]);
        assert!(fs::metadata(&light).unwrap().len() > 0);
        assert!(fs::metadata(&dark).unwrap().len() > 0);
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cli_usage_source_target_and_internal_exit_codes_are_distinct() {
    let root = temp_root("exit-codes");
    let source = write_fixture(&root).canonicalize().unwrap();
    let target = root.join("result.html");

    let usage = run_markhola(&["version", "--overwrite"]);
    assert_eq!(usage.status.code(), Some(EXIT_USAGE));

    let source_error = run_markhola(&[
        "export-html",
        "--source=relative.md",
        &format!("--target={}", target.display()),
    ]);
    assert_eq!(source_error.status.code(), Some(EXIT_SOURCE));

    fs::write(&target, "existing").unwrap();
    let target_error = run_markhola(&[
        "export-html",
        &format!("--source={}", source.display()),
        &format!("--target={}", target.display()),
    ]);
    assert_eq!(target_error.status.code(), Some(EXIT_TARGET));

    let internal = run_if_requested(&args(&[
        "export-png",
        &format!("--source={}", source.display()),
        &format!("--target={}", root.join("result.png").display()),
    ]));
    assert_eq!(internal, Some(EXIT_INTERNAL));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn symlink_and_path_swap_like_targets_fail_closed_without_source_mutation() {
    use std::os::unix::fs::symlink;

    let root = temp_root("target-safety").canonicalize().unwrap();
    let source = write_fixture(&root).canonicalize().unwrap();
    let symlink_target = root.join("symlink.html");
    let real_target = root.join("real.html");
    let before = fs::read_to_string(&source).unwrap();
    fs::write(&real_target, "real").unwrap();
    symlink(&real_target, &symlink_target).unwrap();

    let symlink_error = run_markhola(&[
        "export-html",
        &format!("--source={}", source.display()),
        &format!("--target={}", symlink_target.display()),
        "--json",
    ]);
    assert_eq!(symlink_error.status.code(), Some(EXIT_TARGET));
    let json: serde_json::Value = serde_json::from_slice(&symlink_error.stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["error_code"], "unsafe_output_path");
    assert_eq!(fs::read_to_string(&source).unwrap(), before);
    assert_eq!(fs::read_to_string(&real_target).unwrap(), "real");

    let app_bundle_target = root.join("Fake.app").join("output.html");
    fs::create_dir_all(app_bundle_target.parent().unwrap()).unwrap();
    let bundle_error = run_markhola(&[
        "export-html",
        &format!("--source={}", source.display()),
        &format!("--target={}", app_bundle_target.display()),
        "--json",
    ]);
    assert_eq!(bundle_error.status.code(), Some(EXIT_TARGET));
    let bundle_json: serde_json::Value = serde_json::from_slice(&bundle_error.stdout).unwrap();
    assert_eq!(bundle_json["error_code"], "unsafe_output_path");

    fs::remove_dir_all(root).unwrap();
}
