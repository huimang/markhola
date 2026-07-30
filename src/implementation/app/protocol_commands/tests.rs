use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

use crate::document::ActiveDocument;
use crate::workspace::DocumentWorkspace;

use super::{CACHE_LIMIT, ProtocolCommandRuntime};
use crate::app::implementation::protocol_transport::ProtocolIdentity;

const INSTANCE_ID: &str = "test-instance";
const TOKEN: &str = "test-token";
const REGISTRY_LOCK: &str = "/tmp/markhola-export-registry-test-lock";

struct RegistryGuard;

impl RegistryGuard {
    fn acquire() -> Self {
        loop {
            match fs::create_dir(REGISTRY_LOCK) {
                Ok(()) => return Self,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("failed to acquire export registry test lock: {error}"),
            }
        }
    }
}

impl Drop for RegistryGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir(REGISTRY_LOCK);
    }
}

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
    assert_eq!(
        response(&runtime.handle(
            br#"{"request_id":"unknown-field","instance_token":"test-token","command":"get_instance_state","target":{"instance_id":"test-instance"},"extra":true}"#,
            &workspace,
        ))["error_code"],
        "invalid_request"
    );
    assert_eq!(
        response(&runtime.handle(
            br#"{"request_id":"bad-target","instance_token":"test-token","command":"get_document_state","target":{"instance_id":"test-instance","document_id":7,"expected_version":1,"unexpected":1}}"#,
            &workspace,
        ))["error_code"],
        "invalid_request"
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
fn exact_document_state_tracks_version_after_local_mutation() {
    let mut workspace = workspace();
    let mut runtime = runtime();
    let stale_before = response(&runtime.handle(
        &request(
            "before-mutate",
            "get_document_state",
            json!({
                "instance_id": INSTANCE_ID,
                "document_id": 7,
                "expected_version": 1,
            }),
        ),
        &workspace,
    ));
    assert_eq!(stale_before["result"]["document"]["version"], 1);

    workspace
        .active_document_mut()
        .unwrap()
        .update_markdown("# Protocol\nchanged".to_string());

    let stale = response(&runtime.handle(
        &request(
            "after-mutate-stale",
            "get_document_state",
            json!({
                "instance_id": INSTANCE_ID,
                "document_id": 7,
                "expected_version": 1,
            }),
        ),
        &workspace,
    ));
    assert_eq!(stale["error_code"], "stale_document_version");

    let fresh = response(&runtime.handle(
        &request(
            "after-mutate-fresh",
            "get_document_state",
            json!({
                "instance_id": INSTANCE_ID,
                "document_id": 7,
                "expected_version": 2,
            }),
        ),
        &workspace,
    ));
    assert_eq!(fresh["result"]["document"]["version"], 2);
    assert_eq!(fresh["result"]["document"]["dirty"], true);
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
fn reports_failed_status_for_cached_error_responses() {
    let workspace = workspace();
    let mut runtime = runtime();
    runtime.handle(
        &request(
            "missing-doc",
            "get_document_state",
            json!({ "instance_id": INSTANCE_ID, "document_id": 7 }),
        ),
        &workspace,
    );

    let status_payload = json!({
        "request_id": "status-failed",
        "instance_token": TOKEN,
        "command": "get_request_status",
        "target": target(),
        "request": { "request_id": "missing-doc" },
    });
    let status =
        response(&runtime.handle(&serde_json::to_vec(&status_payload).unwrap(), &workspace));
    assert_eq!(status["result"]["status"], "failed");

    let cancel_payload = json!({
        "request_id": "cancel-failed",
        "instance_token": TOKEN,
        "command": "cancel_request",
        "target": target(),
        "request": { "request_id": "missing-doc" },
    });
    let cancel =
        response(&runtime.handle(&serde_json::to_vec(&cancel_payload).unwrap(), &workspace));
    assert_eq!(cancel["result"]["status"], "too_late");
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
    let before_snapshot = workspace.active_document_snapshot().unwrap();
    for command in [
        "open_document",
        "replace_document_content",
        "set_document_mode",
        "wait_render_ready",
        "save_document",
        "save_document_as",
    ] {
        let result = response(&runtime.handle(
            &request(&format!("not-ready-{command}"), command, target()),
            &workspace,
        ));
        assert_eq!(result["error_code"], "command_not_ready");

        let status_payload = json!({
            "request_id": format!("status-{command}"),
            "instance_token": TOKEN,
            "command": "get_request_status",
            "target": target(),
            "request": { "request_id": format!("not-ready-{command}") },
        });
        let status =
            response(&runtime.handle(&serde_json::to_vec(&status_payload).unwrap(), &workspace));
        assert_eq!(status["result"]["status"], "failed");
    }
    let after_snapshot = workspace.active_document_snapshot().unwrap();
    assert_eq!(after_snapshot.document_id, before_snapshot.document_id);
    assert_eq!(after_snapshot.version, before_snapshot.version);
    assert_eq!(
        after_snapshot.content_sha256,
        before_snapshot.content_sha256
    );
}

#[test]
fn html_export_uses_exact_document_identity_and_explicit_output() {
    let _guard = RegistryGuard::acquire();
    let workspace = workspace();
    let mut runtime = runtime();
    let output = std::env::temp_dir().join(format!(
        "markhola-protocol-html-{}-{}.html",
        std::process::id(),
        CACHE_LIMIT,
    ));
    let _ = std::fs::remove_file(&output);
    let payload = json!({
        "request_id": "export-html",
        "instance_token": TOKEN,
        "command": "export_html",
        "target": {
            "instance_id": INSTANCE_ID,
            "document_id": 7,
            "expected_version": 1,
        },
        "output": {
            "path": output,
            "overwrite": false,
        },
    });
    let result = response(&runtime.handle(&serde_json::to_vec(&payload).unwrap(), &workspace));

    assert_eq!(result["ok"], true);
    assert_eq!(result["result"]["document_id"], 7);
    assert_eq!(result["result"]["sha256"].as_str().unwrap().len(), 64);
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("Protocol")
    );
    std::fs::remove_file(output).unwrap();
}

#[test]
fn export_rejects_format_mismatch_before_rendering() {
    let _guard = RegistryGuard::acquire();
    let workspace = workspace();
    let mut runtime = runtime();
    let payload = json!({
        "request_id": "png-wrong-extension",
        "instance_token": TOKEN,
        "command": "export_png",
        "target": {
            "instance_id": INSTANCE_ID,
            "document_id": 7,
            "expected_version": 1,
        },
        "output": {
            "path": std::env::temp_dir().join("markhola-wrong-extension.pdf"),
            "overwrite": false,
        },
    });
    let result = response(&runtime.handle(&serde_json::to_vec(&payload).unwrap(), &workspace));

    assert_eq!(result["error_code"], "invalid_output_extension");
}

#[test]
fn export_requests_require_strict_output_schema_and_exact_document_identity() {
    let _guard = RegistryGuard::acquire();
    let workspace = workspace();
    let mut runtime = runtime();

    let missing_output = response(&runtime.handle(
        &request(
            "missing-output",
            "export_html",
            json!({
                "instance_id": INSTANCE_ID,
                "document_id": 7,
                "expected_version": 1,
            }),
        ),
        &workspace,
    ));
    assert_eq!(missing_output["error_code"], "missing_output");

    let missing_version = response(&runtime.handle(
        &request(
            "missing-export-version",
            "export_html",
            json!({
                "instance_id": INSTANCE_ID,
                "document_id": 7,
            }),
        ),
        &workspace,
    ));
    assert_eq!(missing_version["error_code"], "missing_document_version");

    let unknown_output_field = response(&runtime.handle(
        br#"{"request_id":"bad-output","instance_token":"test-token","command":"export_html","target":{"instance_id":"test-instance","document_id":7,"expected_version":1},"output":{"path":"/tmp/protocol-bad-output.html","overwrite":false,"extra":true}}"#,
        &workspace,
    ));
    assert_eq!(unknown_output_field["error_code"], "invalid_request");
}

#[test]
fn export_requests_cache_completed_and_failed_statuses_with_exact_identity() {
    let _guard = RegistryGuard::acquire();
    let workspace = workspace();
    let mut runtime = runtime();
    let output = std::env::temp_dir().join(format!(
        "markhola-protocol-status-{}-{}.html",
        std::process::id(),
        CACHE_LIMIT + 1,
    ));
    let _ = std::fs::remove_file(&output);

    let success_payload = json!({
        "request_id": "export-status-success",
        "instance_token": TOKEN,
        "command": "export_html",
        "target": {
            "instance_id": INSTANCE_ID,
            "document_id": 7,
            "expected_version": 1,
        },
        "output": {
            "path": output,
            "overwrite": false,
        },
    });
    let success =
        response(&runtime.handle(&serde_json::to_vec(&success_payload).unwrap(), &workspace));
    assert_eq!(success["ok"], true);
    assert_eq!(success["result"]["instance_id"], INSTANCE_ID);
    assert_eq!(success["result"]["document_id"], 7);
    assert_eq!(success["result"]["document_version"], 1);

    let success_status = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-status-check",
                "instance_token": TOKEN,
                "command": "get_request_status",
                "target": target(),
                "request": { "request_id": "export-status-success" },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(success_status["result"]["status"], "completed");

    let success_cancel = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-cancel-check",
                "instance_token": TOKEN,
                "command": "cancel_request",
                "target": target(),
                "request": { "request_id": "export-status-success" },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(success_cancel["result"]["status"], "too_late");

    let replay =
        response(&runtime.handle(&serde_json::to_vec(&success_payload).unwrap(), &workspace));
    assert_eq!(replay, success);

    let replay_status = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-replay-status-check",
                "instance_token": TOKEN,
                "command": "get_request_status",
                "target": target(),
                "request": { "request_id": "export-status-success" },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(replay_status["result"]["status"], "completed");

    let output_exists = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-status-output-exists",
                "instance_token": TOKEN,
                "command": "export_html",
                "target": {
                    "instance_id": INSTANCE_ID,
                    "document_id": 7,
                    "expected_version": 1,
                },
                "output": {
                    "path": output,
                    "overwrite": false,
                },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(output_exists["error_code"], "output_exists");

    let output_exists_status = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-output-exists-status-check",
                "instance_token": TOKEN,
                "command": "get_request_status",
                "target": target(),
                "request": { "request_id": "export-status-output-exists" },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(output_exists_status["result"]["status"], "failed");

    let overwrite_failure = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-status-failure",
                "instance_token": TOKEN,
                "command": "export_html",
                "target": {
                    "instance_id": INSTANCE_ID,
                    "document_id": 7,
                    "expected_version": 2,
                },
                "output": {
                    "path": output,
                    "overwrite": true,
                },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(overwrite_failure["error_code"], "stale_document_version");

    let failed_status = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-status-after-failure",
                "instance_token": TOKEN,
                "command": "get_request_status",
                "target": target(),
                "request": { "request_id": "export-status-failure" },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(failed_status["result"]["status"], "failed");

    let failed_cancel = response(
        &runtime.handle(
            &serde_json::to_vec(&json!({
                "request_id": "export-cancel-after-failure",
                "instance_token": TOKEN,
                "command": "cancel_request",
                "target": target(),
                "request": { "request_id": "export-status-failure" },
            }))
            .unwrap(),
            &workspace,
        ),
    );
    assert_eq!(failed_cancel["result"]["status"], "too_late");

    std::fs::remove_file(output).unwrap();
}
