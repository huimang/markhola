use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::export_service::{self, CancelOutcome};

const CACHE_LIMIT: usize = 256;
const CANCELLATION_WAIT: Duration = Duration::from_secs(61);

pub(super) struct ExportControl {
    instance_id: String,
    cache: Mutex<ControlCache>,
}

#[derive(Default)]
struct ControlCache {
    entries: HashMap<String, CachedResponse>,
    order: VecDeque<String>,
}

struct CachedResponse {
    payload: Vec<u8>,
    response: Vec<u8>,
}

#[derive(Deserialize)]
struct CommandProbe<'a> {
    command: &'a str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlRequest {
    request_id: String,
    #[serde(rename = "instance_token")]
    _instance_token: String,
    command: ControlCommand,
    target: ControlTarget,
    request: RequestReference,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ControlCommand {
    GetRequestStatus,
    CancelRequest,
}

impl ControlCommand {
    fn name(self) -> &'static str {
        match self {
            Self::GetRequestStatus => "get_request_status",
            Self::CancelRequest => "cancel_request",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlTarget {
    instance_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestReference {
    request_id: String,
}

impl ExportControl {
    pub(super) fn new(instance_id: String) -> Self {
        Self {
            instance_id,
            cache: Mutex::new(ControlCache::default()),
        }
    }

    pub(super) fn handle(&self, payload: &[u8]) -> Option<Vec<u8>> {
        let probe = serde_json::from_slice::<CommandProbe<'_>>(payload).ok()?;
        if !matches!(probe.command, "get_request_status" | "cancel_request") {
            return None;
        }
        let request = match serde_json::from_slice::<ControlRequest>(payload) {
            Ok(request) => request,
            Err(_) => {
                return Some(encode(error(
                    "",
                    probe.command,
                    "invalid_request",
                    "Malformed request.",
                )));
            }
        };
        if request.request_id.is_empty() {
            return Some(encode(error(
                "",
                request.command.name(),
                "invalid_request",
                "Missing request id.",
            )));
        }
        if let Some(response) = self.cached_response(&request.request_id, payload) {
            return Some(response);
        }
        if export_service::export_status(&request.request.request_id).is_none() {
            return None;
        }
        let response = if request.target.instance_id != self.instance_id {
            error(
                &request.request_id,
                request.command.name(),
                "instance_mismatch",
                "The selected application instance does not match.",
            )
        } else {
            match request.command {
                ControlCommand::GetRequestStatus => status_response(&request),
                ControlCommand::CancelRequest => cancel_response(&request),
            }
        };
        let response = encode(response);
        self.remember(request.request_id, payload.to_vec(), response.clone());
        Some(response)
    }

    fn cached_response(&self, request_id: &str, payload: &[u8]) -> Option<Vec<u8>> {
        let cache = self.cache.lock().unwrap_or_else(|error| error.into_inner());
        cache.entries.get(request_id).map(|cached| {
            if cached.payload == payload {
                cached.response.clone()
            } else {
                encode(error(
                    request_id,
                    "",
                    "request_id_conflict",
                    "The request id was already used for a different request.",
                ))
            }
        })
    }

    fn remember(&self, request_id: String, payload: Vec<u8>, response: Vec<u8>) {
        let mut cache = self.cache.lock().unwrap_or_else(|error| error.into_inner());
        while cache.order.len() >= CACHE_LIMIT {
            if let Some(expired) = cache.order.pop_front() {
                cache.entries.remove(&expired);
            }
        }
        cache.order.push_back(request_id.clone());
        cache
            .entries
            .insert(request_id, CachedResponse { payload, response });
    }
}

fn status_response(request: &ControlRequest) -> Value {
    let Some(status) = export_service::export_status(&request.request.request_id) else {
        return error(
            &request.request_id,
            request.command.name(),
            "request_not_found",
            "The request is not in the process-lifetime cache.",
        );
    };
    success(
        request,
        json!({
            "target_request_id": request.request.request_id,
            "status": status.name(),
        }),
    )
}

fn cancel_response(request: &ControlRequest) -> Value {
    match export_service::cancel_export_and_wait(&request.request.request_id, CANCELLATION_WAIT) {
        CancelOutcome::Cancelled => success(
            request,
            json!({
                "target_request_id": request.request.request_id,
                "status": "cancelled",
            }),
        ),
        CancelOutcome::TooLate => success(
            request,
            json!({
                "target_request_id": request.request.request_id,
                "status": "too_late",
            }),
        ),
        CancelOutcome::NotFound => error(
            &request.request_id,
            request.command.name(),
            "request_not_found",
            "The request is not in the process-lifetime cache.",
        ),
        CancelOutcome::TimedOut => error(
            &request.request_id,
            request.command.name(),
            "cancellation_timeout",
            "The export did not reach a terminal state before the cancellation timeout.",
        ),
    }
}

fn success(request: &ControlRequest, result: Value) -> Value {
    json!({
        "request_id": request.request_id,
        "ok": true,
        "command": request.command.name(),
        "result": result,
    })
}

fn error(request_id: &str, command: &str, code: &str, message: &str) -> Value {
    json!({
        "request_id": request_id,
        "ok": false,
        "command": command,
        "error_code": code,
        "message": message,
    })
}

fn encode(value: Value) -> Vec<u8> {
    serde_json::to_vec(&value)
        .unwrap_or_else(|_| br#"{"ok":false,"error_code":"internal_error"}"#.to_vec())
}

#[cfg(test)]
#[path = "control/tests.rs"]
mod tests;
