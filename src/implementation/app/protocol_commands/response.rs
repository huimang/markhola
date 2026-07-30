use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::document::{ActiveDocument, DocumentMode};

use super::schema::Request;

pub(super) fn document_value(document: &ActiveDocument, active_id: Option<u64>) -> Value {
    json!({
        "document_id": document.id(),
        "path": if document.is_draft() {
            Value::Null
        } else {
            json!(document.canonical_path())
        },
        "active": active_id == Some(document.id()),
        "mode": match document.mode() {
            DocumentMode::Readonly => "readonly",
            DocumentMode::Writable => "edit",
        },
        "dirty": document.is_dirty(),
        "version": document.version(),
        "content_sha256": document.content_sha256(),
        "render_generation": document.render_generation(),
        "errors": [],
    })
}

pub(super) fn success(request: &Request, result: Value) -> Value {
    json!({
        "request_id": request.request_id,
        "ok": true,
        "command": request.command.name(),
        "result": result,
    })
}

pub(super) fn error(request_id: &str, command: &str, code: &str, message: &str) -> Value {
    json!({
        "request_id": request_id,
        "ok": false,
        "command": command,
        "error_code": code,
        "message": message,
    })
}

pub(super) fn encode(value: Value) -> Vec<u8> {
    serde_json::to_vec(&value)
        .unwrap_or_else(|_| br#"{"ok":false,"error_code":"internal_error"}"#.to_vec())
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
