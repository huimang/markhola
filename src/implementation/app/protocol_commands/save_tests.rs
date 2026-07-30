use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

use crate::document::ActiveDocument;
use crate::workspace::DocumentWorkspace;

use super::ProtocolCommandRuntime;
use crate::app::implementation::protocol_transport::ProtocolIdentity;

const INSTANCE_ID: &str = "save-instance";
const TOKEN: &str = "save-token";
static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

fn root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "markhola-protocol-save-{label}-{}-{}",
        std::process::id(),
        NEXT_TEST.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn workspace(path: &std::path::Path, current: &str) -> DocumentWorkspace {
    fs::write(path, "# Source").unwrap();
    let mut document = ActiveDocument::open_with_id(
        7,
        path.to_path_buf(),
        "# Source".to_string(),
        crate::file_io::directory_base_url(path).unwrap(),
    );
    document.toggle_mode();
    document.update_markdown(current.to_string());
    let mut workspace = DocumentWorkspace::new();
    workspace.open_document(document);
    workspace
}

fn runtime() -> ProtocolCommandRuntime {
    ProtocolCommandRuntime::new(ProtocolIdentity::for_test(INSTANCE_ID, TOKEN))
}

#[test]
fn save_document_preserves_identity_and_reports_completed_too_late() {
    let root = root("existing");
    let source = root.join("source.md");
    let mut workspace = workspace(&source, "# Current memory");
    let version = workspace.document_by_id(7).unwrap().version();
    let mut runtime = runtime();
    let payload = request("save-existing", "save_document", version, None);

    let success = response(runtime.handle_mut(&payload, &mut workspace));
    assert_eq!(success["ok"], true);
    assert_eq!(success["result"]["document_id"], 7);
    assert_eq!(
        success["result"]["path"],
        source.canonicalize().unwrap().to_string_lossy().as_ref()
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "# Current memory");
    assert_eq!(
        response(runtime.handle_mut(&payload, &mut workspace)),
        success
    );

    let status = response(runtime.handle_mut(
        &control("save-status", "get_request_status", "save-existing"),
        &mut workspace,
    ));
    assert_eq!(status["result"]["status"], "completed");
    let cancel = response(runtime.handle_mut(
        &control("save-cancel", "cancel_request", "save-existing"),
        &mut workspace,
    ));
    assert_eq!(cancel["result"]["status"], "too_late");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_as_preserves_source_and_transitions_path_after_success() {
    let root = root("save-as");
    let source = root.join("source.md");
    let target = root.join("copy.md");
    let mut workspace = workspace(&source, "# Unsaved current memory");
    let version = workspace.document_by_id(7).unwrap().version();
    let mut runtime = runtime();
    let payload = request(
        "save-as",
        "save_document_as",
        version,
        Some((&target, false)),
    );

    let success = response(runtime.handle_mut(&payload, &mut workspace));
    assert_eq!(success["ok"], true);
    assert_eq!(fs::read_to_string(&source).unwrap(), "# Source");
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "# Unsaved current memory"
    );
    let document = workspace.document_by_id(7).unwrap();
    assert_eq!(document.canonical_path(), target.canonicalize().unwrap());
    assert_eq!(success["result"]["document_version"], document.version());
    assert_eq!(
        success["result"]["content_sha256"],
        document.content_sha256()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_commands_fail_closed_for_stale_readonly_and_external_state() {
    let root = root("failures");
    let source = root.join("source.md");
    let mut workspace = workspace(&source, "# Current memory");
    let version = workspace.document_by_id(7).unwrap().version();
    let mut runtime = runtime();

    let stale = response(runtime.handle_mut(
        &request("stale-save", "save_document", version + 1, None),
        &mut workspace,
    ));
    assert_eq!(stale["error_code"], "stale_document_version");

    fs::write(&source, "# External").unwrap();
    let external = response(runtime.handle_mut(
        &request("external-save", "save_document", version, None),
        &mut workspace,
    ));
    assert_eq!(external["error_code"], "external_source_changed");
    assert_eq!(fs::read_to_string(&source).unwrap(), "# External");

    workspace.document_by_id_mut(7).unwrap().toggle_mode();
    let readonly_version = workspace.document_by_id(7).unwrap().version();
    let readonly = response(runtime.handle_mut(
        &request(
            "readonly-save-as",
            "save_document_as",
            readonly_version,
            Some((&root.join("readonly.md"), false)),
        ),
        &mut workspace,
    ));
    assert_eq!(readonly["error_code"], "document_readonly");
    assert!(!root.join("readonly.md").exists());
    fs::remove_dir_all(root).unwrap();
}

fn request(
    request_id: &str,
    command: &str,
    version: u64,
    output: Option<(&std::path::Path, bool)>,
) -> Vec<u8> {
    let mut value = json!({
        "request_id": request_id,
        "instance_token": TOKEN,
        "command": command,
        "target": {
            "instance_id": INSTANCE_ID,
            "document_id": 7,
            "expected_version": version,
        },
    });
    if let Some((path, overwrite)) = output {
        value["output"] = json!({ "path": path, "overwrite": overwrite });
    }
    serde_json::to_vec(&value).unwrap()
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
