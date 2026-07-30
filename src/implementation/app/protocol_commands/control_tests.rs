use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

use crate::document::ActiveDocument;
use crate::workspace::DocumentWorkspace;

use super::ProtocolCommandRuntime;
use crate::app::implementation::asset_access;
use crate::app::implementation::protocol_transport::ProtocolIdentity;

const INSTANCE_ID: &str = "control-instance";
const TOKEN: &str = "control-token";
static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

#[test]
fn opens_only_safe_markdown_without_changing_active_document() {
    let root = root("open");
    let original = root.join("original.md");
    let opened = root.join("opened.md");
    fs::write(&original, "# Original").unwrap();
    fs::write(&opened, "# Opened").unwrap();
    let mut workspace = workspace(&original);
    let active_before = workspace.active_document_id();
    let registry = asset_access::new_registry();
    let mut runtime = runtime();

    let result = response(runtime.handle_app_mut(
        &payload(
            "open",
            "open_document",
            json!({"instance_id": INSTANCE_ID}),
            json!({"path": opened}),
        ),
        &mut workspace,
        &registry,
    ));

    assert_eq!(result["ok"], true);
    assert_eq!(workspace.active_document_id(), active_before);
    assert_eq!(result["result"]["document"]["active"], false);
    assert_eq!(workspace.document_ids().len(), 2);

    let symlink = root.join("linked.md");
    std::os::unix::fs::symlink(&opened, &symlink).unwrap();
    let rejected = response(runtime.handle_app_mut(
        &payload(
            "open-link",
            "open_document",
            json!({"instance_id": INSTANCE_ID}),
            json!({"path": symlink}),
        ),
        &mut workspace,
        &registry,
    ));
    assert_eq!(rejected["error_code"], "unsafe_document_path");

    let replay = response(runtime.handle_app_mut(
        &payload(
            "open",
            "open_document",
            json!({"instance_id": INSTANCE_ID}),
            json!({"path": opened}),
        ),
        &mut workspace,
        &registry,
    ));
    assert_eq!(replay, result);

    let conflict = response(runtime.handle_app_mut(
        &payload(
            "open",
            "open_document",
            json!({"instance_id": INSTANCE_ID}),
            json!({"path": original}),
        ),
        &mut workspace,
        &registry,
    ));
    assert_eq!(conflict["error_code"], "request_id_conflict");

    let relative = response(runtime.handle_app_mut(
        &payload(
            "open-relative",
            "open_document",
            json!({"instance_id": INSTANCE_ID}),
            json!({"path": "opened.md"}),
        ),
        &mut workspace,
        &registry,
    ));
    assert_eq!(relative["error_code"], "invalid_document_path");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replaces_exact_writable_clean_content_and_rejects_stale_or_dirty() {
    let root = root("replace");
    let source = root.join("source.md");
    let mut workspace = workspace(&source);
    let document = workspace.active_document_mut().unwrap();
    document.toggle_mode();
    let version = document.version();
    let mut runtime = runtime();

    let success = response(runtime.handle_mut(
        &payload(
            "replace",
            "replace_document_content",
            target(1, version),
            json!({"content": "# Replaced"}),
        ),
        &mut workspace,
    ));
    assert_eq!(success["ok"], true);
    assert_eq!(
        success["result"]["document"]["content_sha256"],
        workspace.active_document().unwrap().content_sha256()
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "# Original");

    let replay = response(runtime.handle_mut(
        &payload(
            "replace",
            "replace_document_content",
            target(1, version),
            json!({"content": "# Replaced"}),
        ),
        &mut workspace,
    ));
    assert_eq!(replay, success);

    let conflict = response(runtime.handle_mut(
        &payload(
            "replace",
            "replace_document_content",
            target(1, workspace.active_document().unwrap().version()),
            json!({"content": "# Conflict"}),
        ),
        &mut workspace,
    ));
    assert_eq!(conflict["error_code"], "request_id_conflict");

    let dirty = response(runtime.handle_mut(
        &payload(
            "replace-dirty",
            "replace_document_content",
            target(1, workspace.active_document().unwrap().version()),
            json!({"content": "# Again"}),
        ),
        &mut workspace,
    ));
    assert_eq!(dirty["error_code"], "document_dirty");
    assert_eq!(
        workspace.active_document().unwrap().markdown(),
        "# Replaced",
        "failed replacement must not mutate the active document"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mode_and_render_wait_are_bound_to_exact_version_and_generation() {
    let root = root("render");
    let source = root.join("source.md");
    let mut workspace = workspace(&source);
    let version = workspace.active_document().unwrap().version();
    let mut runtime = runtime();

    let mode = response(runtime.handle_mut(
        &payload(
            "edit",
            "set_document_mode",
            target(1, version),
            json!({"mode": "edit"}),
        ),
        &mut workspace,
    ));
    assert_eq!(mode["result"]["document"]["mode"], "edit");
    let document_version = workspace.active_document().unwrap().version();
    let render_generation = workspace.active_document().unwrap().render_generation();
    let ready = response(runtime.handle_mut(
        &payload(
            "ready",
            "wait_render_ready",
            target(1, document_version),
            json!({"render_generation": render_generation}),
        ),
        &mut workspace,
    ));
    assert_eq!(ready["result"]["status"], "ready");

    let mismatch = response(runtime.handle_mut(
        &payload(
            "not-ready",
            "wait_render_ready",
            target(1, document_version),
            json!({"render_generation": render_generation + 1}),
        ),
        &mut workspace,
    ));
    assert_eq!(mismatch["error_code"], "render_generation_mismatch");

    let stale = response(runtime.handle_mut(
        &payload(
            "stale-ready",
            "wait_render_ready",
            target(1, document_version - 1),
            json!({"render_generation": render_generation}),
        ),
        &mut workspace,
    ));
    assert_eq!(stale["error_code"], "stale_document_version");

    let missing_mode = response(runtime.handle_mut(
        &payload(
            "missing-mode",
            "set_document_mode",
            target(1, document_version),
            json!({}),
        ),
        &mut workspace,
    ));
    assert_eq!(missing_mode["error_code"], "missing_document_mode");

    let readonly = response(runtime.handle_mut(
        &payload(
            "readonly",
            "set_document_mode",
            target(1, document_version),
            json!({"mode": "readonly"}),
        ),
        &mut workspace,
    ));
    assert_eq!(readonly["ok"], true);
    assert_eq!(readonly["result"]["document"]["mode"], "readonly");

    let status = response(runtime.handle_mut(
        &control("ready-status", "get_request_status", "ready"),
        &mut workspace,
    ));
    assert_eq!(status["result"]["status"], "completed");
    let cancel = response(runtime.handle_mut(
        &control("ready-cancel", "cancel_request", "ready"),
        &mut workspace,
    ));
    assert_eq!(cancel["result"]["status"], "too_late");
    fs::remove_dir_all(root).unwrap();
}

fn workspace(path: &std::path::Path) -> DocumentWorkspace {
    fs::write(path, "# Original").unwrap();
    let document = ActiveDocument::open_with_id(
        1,
        path.to_path_buf(),
        "# Original".to_string(),
        crate::file_io::directory_base_url(path).unwrap(),
    );
    let mut workspace = DocumentWorkspace::new();
    workspace.open_document(document);
    workspace
}

fn runtime() -> ProtocolCommandRuntime {
    ProtocolCommandRuntime::new(ProtocolIdentity::for_test(INSTANCE_ID, TOKEN))
}

fn target(document_id: u64, version: u64) -> Value {
    json!({
        "instance_id": INSTANCE_ID,
        "document_id": document_id,
        "expected_version": version,
    })
}

fn payload(request_id: &str, command: &str, target: Value, extra: Value) -> Vec<u8> {
    let mut request = json!({
        "request_id": request_id,
        "instance_token": TOKEN,
        "command": command,
        "target": target,
    });
    for (key, value) in extra.as_object().unwrap() {
        request[key] = value.clone();
    }
    serde_json::to_vec(&request).unwrap()
}

fn control(request_id: &str, command: &str, target_request_id: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "request_id": request_id,
        "instance_token": TOKEN,
        "command": command,
        "target": { "instance_id": INSTANCE_ID },
        "request": { "request_id": target_request_id },
    }))
    .unwrap()
}

fn response(bytes: Vec<u8>) -> Value {
    serde_json::from_slice(&bytes).unwrap()
}

fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "markhola-control-{label}-{}-{}",
        std::process::id(),
        NEXT_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
