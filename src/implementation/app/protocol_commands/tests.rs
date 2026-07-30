use std::path::PathBuf;

use serde_json::{Value, json};

use crate::document::ActiveDocument;
use crate::workspace::DocumentWorkspace;

use super::{CACHE_LIMIT, ProtocolCommandRuntime};
use crate::app::implementation::protocol_transport::ProtocolIdentity;

const INSTANCE_ID: &str = "test-instance";
const TOKEN: &str = "test-token";

fn runtime() -> ProtocolCommandRuntime {
    ProtocolCommandRuntime::new(ProtocolIdentity::for_test(INSTANCE_ID, TOKEN))
}

fn workspace() -> DocumentWorkspace {
    let mut workspace = DocumentWorkspace::new();
    workspace.open_document(ActiveDocument::open_with_id(
        7,
        PathBuf::from("/tmp/protocol document.md"),
        "# Protocol\nbody".to_string(),
        "file:///tmp/".to_string(),
    ));
    workspace
}

fn request(request_id: &str, command: &str, target: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "request_id": request_id,
        "instance_token": TOKEN,
        "command": command,
        "target": target,
    }))
    .unwrap()
}

fn target() -> Value {
    json!({ "instance_id": INSTANCE_ID })
}

fn response(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

#[test]
fn rejects_malformed_unknown_and_wrong_instance_requests() {
    let workspace = workspace();
    let mut runtime = runtime();

    assert_eq!(
        response(&runtime.handle(b"{", &workspace))["error_code"],
        "invalid_request"
    );
    assert_eq!(
        response(&runtime.handle(&request("unknown", "run_shell", target()), &workspace))["error_code"],
        "invalid_request"
    );
    assert_eq!(
        response(&runtime.handle(
            &request(
                "wrong-instance",
                "get_instance_state",
                json!({ "instance_id": "other" }),
            ),
            &workspace,
        ))["error_code"],
        "instance_mismatch"
    );
}

#[test]
fn reports_exact_instance_and_document_state() {
    let workspace = workspace();
    let mut runtime = runtime();
    let instance = response(&runtime.handle(
        &request("instance", "get_instance_state", target()),
        &workspace,
    ));
    assert_eq!(instance["result"]["instance_id"], INSTANCE_ID);
    assert_eq!(instance["result"]["document_count"], 1);

    let list = response(&runtime.handle(
        &request("list", "list_document_state", target()),
        &workspace,
    ));
    let document = &list["result"]["documents"][0];
    assert_eq!(document["document_id"], 7);
    assert_eq!(document["version"], 1);
    assert_eq!(document["mode"], "readonly");
    assert_eq!(document["content_sha256"].as_str().unwrap().len(), 64);

    let exact = response(&runtime.handle(
        &request(
            "document",
            "get_document_state",
            json!({
                "instance_id": INSTANCE_ID,
                "document_id": 7,
                "expected_version": 1,
            }),
        ),
        &workspace,
    ));
    assert_eq!(exact["result"]["document"]["document_id"], 7);
}

#[test]
fn rejects_missing_and_stale_document_identity() {
    let workspace = workspace();
    let mut runtime = runtime();

    let missing_version = response(&runtime.handle(
        &request(
            "missing-version",
            "get_document_state",
            json!({ "instance_id": INSTANCE_ID, "document_id": 7 }),
        ),
        &workspace,
    ));
    assert_eq!(missing_version["error_code"], "missing_document_version");

    let stale = response(&runtime.handle(
        &request(
            "stale",
            "get_document_state",
            json!({
                "instance_id": INSTANCE_ID,
                "document_id": 7,
                "expected_version": 2,
            }),
        ),
        &workspace,
    ));
    assert_eq!(stale["error_code"], "stale_document_version");
}

#[test]
fn replays_identical_requests_and_rejects_request_id_reuse() {
    let workspace = workspace();
    let mut runtime = runtime();
    let original = request("duplicate", "get_instance_state", target());

    let first = runtime.handle(&original, &workspace);
    assert_eq!(runtime.handle(&original, &workspace), first);

    let conflict = response(&runtime.handle(
        &request("duplicate", "list_document_state", target()),
        &workspace,
    ));
    assert_eq!(conflict["error_code"], "request_id_conflict");
}

#[test]
fn reports_completed_status_and_cancel_too_late() {
    let workspace = workspace();
    let mut runtime = runtime();
    runtime.handle(
        &request("completed", "get_instance_state", target()),
        &workspace,
    );

    for (request_id, command, expected) in [
        ("status", "get_request_status", "completed"),
        ("cancel", "cancel_request", "too_late"),
    ] {
        let payload = json!({
            "request_id": request_id,
            "instance_token": TOKEN,
            "command": command,
            "target": target(),
            "request": { "request_id": "completed" },
        });
        let result = response(&runtime.handle(&serde_json::to_vec(&payload).unwrap(), &workspace));
        assert_eq!(result["result"]["status"], expected);
    }
}

#[test]
fn bounds_process_lifetime_request_cache() {
    let workspace = workspace();
    let mut runtime = runtime();
    for index in 0..=CACHE_LIMIT {
        runtime.handle(
            &request(&format!("request-{index}"), "get_instance_state", target()),
            &workspace,
        );
    }

    let payload = json!({
        "request_id": "status-after-eviction",
        "instance_token": TOKEN,
        "command": "get_request_status",
        "target": target(),
        "request": { "request_id": "request-0" },
    });
    let result = response(&runtime.handle(&serde_json::to_vec(&payload).unwrap(), &workspace));
    assert_eq!(result["error_code"], "request_not_found");
}

#[test]
fn allowlisted_side_effects_fail_closed_until_services_connect() {
    let workspace = workspace();
    let mut runtime = runtime();
    for command in [
        "open_document",
        "replace_document_content",
        "set_document_mode",
        "wait_render_ready",
        "export_png",
        "export_pdf",
        "export_html",
        "save_document",
        "save_document_as",
    ] {
        let result = response(&runtime.handle(
            &request(&format!("not-ready-{command}"), command, target()),
            &workspace,
        ));
        assert_eq!(result["error_code"], "command_not_ready");
    }
}
