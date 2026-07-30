use std::io::{Read, Write};
use std::fs;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

use crate::export_service::{self, ExportStatus};

use super::super::{handle_connection, peer};
use super::ExportControl;

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

#[test]
fn socket_control_stays_responsive_without_event_loop_dispatch() {
    let _guard = RegistryGuard::acquire();
    let control = Arc::new(ExportControl::new("instance-a".to_string()));
    let export_id = format!("socket-export-{}", std::process::id());
    export_service::register_queued_export(&export_id);

    assert_eq!(
        paired_request(
            Arc::clone(&control),
            "socket-status-queued",
            "get_request_status",
            &export_id,
        )["result"]["status"],
        "queued"
    );
    let cancellation = export_service::begin_export_cancellation(&export_id);
    assert_eq!(
        paired_request(
            Arc::clone(&control),
            "socket-status-running",
            "get_request_status",
            &export_id,
        )["result"]["status"],
        "running"
    );

    let cancelling_control = Arc::clone(&control);
    let cancelling_id = export_id.clone();
    let waiter = thread::spawn(move || {
        paired_request(
            cancelling_control,
            "socket-cancel",
            "cancel_request",
            &cancelling_id,
        )
    });
    thread::sleep(Duration::from_millis(30));
    assert!(cancellation.is_cancelled());
    assert!(!waiter.is_finished());
    export_service::finish_export(&export_id, ExportStatus::Cancelled);
    assert_eq!(waiter.join().unwrap()["result"]["status"], "cancelled");
    export_service::finish_export_cancellation(&export_id);
}

#[test]
fn observes_export_lifecycle_and_waits_for_cancel_cleanup() {
    let _guard = RegistryGuard::acquire();
    let export_id = format!("active-export-{}", std::process::id());
    let control = std::sync::Arc::new(ExportControl::new("instance-a".to_string()));
    export_service::register_queued_export(&export_id);
    assert_status(&control, "status-queued", &export_id, "queued");

    let cancellation = export_service::begin_export_cancellation(&export_id);
    assert_status(&control, "status-running", &export_id, "running");

    let cancelling = std::sync::Arc::clone(&control);
    let cancelling_id = export_id.clone();
    let waiter = thread::spawn(move || {
        let payload = request("cancel-active", "cancel_request", &cancelling_id);
        response(cancelling.handle(&payload).unwrap())
    });
    thread::sleep(Duration::from_millis(30));
    assert!(cancellation.is_cancelled());
    assert!(!waiter.is_finished(), "cancel must wait for export cleanup");

    export_service::finish_export(&export_id, ExportStatus::Cancelled);
    assert_eq!(waiter.join().unwrap()["result"]["status"], "cancelled");
    assert_status(&control, "status-cancelled", &export_id, "cancelled");
    export_service::finish_export_cancellation(&export_id);
}

#[test]
fn cancel_after_atomic_commit_is_too_late_and_idempotent() {
    let _guard = RegistryGuard::acquire();
    let export_id = format!("committed-export-{}", std::process::id());
    let control = ExportControl::new("instance-a".to_string());
    export_service::register_queued_export(&export_id);
    export_service::begin_export_cancellation(&export_id);
    export_service::finish_export(&export_id, ExportStatus::Completed);

    let payload = request("cancel-committed", "cancel_request", &export_id);
    let first = control.handle(&payload).unwrap();
    let second = control.handle(&payload).unwrap();
    assert_eq!(first, second);
    assert_eq!(response(first)["result"]["status"], "too_late");
    assert_status(&control, "status-completed", &export_id, "completed");
    export_service::finish_export_cancellation(&export_id);
}

fn assert_status(control: &ExportControl, request_id: &str, export_id: &str, expected: &str) {
    let result = response(
        control
            .handle(&request(request_id, "get_request_status", export_id))
            .unwrap(),
    );
    assert_eq!(result["result"]["status"], expected);
}

fn request(request_id: &str, command: &str, export_id: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "request_id": request_id,
        "instance_token": "token-a",
        "command": command,
        "target": { "instance_id": "instance-a" },
        "request": { "request_id": export_id },
    }))
    .unwrap()
}

fn response(bytes: Vec<u8>) -> Value {
    serde_json::from_slice(&bytes).unwrap()
}

fn paired_request(
    control: Arc<ExportControl>,
    request_id: &str,
    command: &str,
    export_id: &str,
) -> Value {
    let (mut client, server) = UnixStream::pair().unwrap();
    let handler = thread::spawn(move || {
        let proxy = Mutex::new(None);
        handle_connection(server, peer::current_uid(), "token-a", &control, &proxy);
    });
    let payload = serde_json::to_vec(&json!({
        "request_id": request_id,
        "instance_token": "token-a",
        "command": command,
        "target": { "instance_id": "instance-a" },
        "request": { "request_id": export_id },
    }))
    .unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client.write_all(&payload).unwrap();
    client.write_all(b"\n").unwrap();
    client.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    handler.join().unwrap();
    serde_json::from_slice(&response).unwrap()
}
