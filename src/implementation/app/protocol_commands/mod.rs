mod response;
mod schema;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, VecDeque};
use std::path::Path;

use serde_json::{Value, json};

use crate::app::{APP_BUILD_PLATFORM, APP_BUILD_TARGET, APP_VERSION};
use crate::export_service::{self, ExportFormat};
use crate::workspace::DocumentWorkspace;

use super::protocol_transport::ProtocolIdentity;
use response::{document_value, encode, error, sha256, success};
use schema::{Command, Request};

const CACHE_LIMIT: usize = 256;

pub(super) struct ProtocolCommandRuntime {
    identity: ProtocolIdentity,
    cache: HashMap<String, CachedRequest>,
    order: VecDeque<String>,
}

struct CachedRequest {
    fingerprint: String,
    response: Vec<u8>,
    status: RequestStatus,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum RequestStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    TooLate,
}

impl RequestStatus {
    fn name(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TooLate => "too_late",
        }
    }
}

impl ProtocolCommandRuntime {
    pub(super) fn new(identity: ProtocolIdentity) -> Self {
        Self {
            identity,
            cache: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub(super) fn handle(&mut self, payload: &[u8], workspace: &DocumentWorkspace) -> Vec<u8> {
        let fingerprint = sha256(payload);
        let request = match serde_json::from_slice::<Request>(payload) {
            Ok(request) => request,
            Err(_) => return encode(error("", "", "invalid_request", "Malformed request.")),
        };
        if request.request_id.is_empty() {
            return encode(error(
                "",
                request.command.name(),
                "invalid_request",
                "Missing request id.",
            ));
        }
        if let Some(cached) = self.cache.get(&request.request_id) {
            if cached.fingerprint == fingerprint {
                return cached.response.clone();
            }
            return encode(error(
                &request.request_id,
                request.command.name(),
                "request_id_conflict",
                "The request id was already used for a different request.",
            ));
        }

        let response = if request.target.instance_id != self.identity.instance_id()
            || request.instance_token != self.identity.exact_instance_token()
        {
            error(
                &request.request_id,
                request.command.name(),
                "instance_mismatch",
                "The selected application instance does not match.",
            )
        } else {
            self.execute(&request, workspace)
        };
        if request.command.is_export() {
            export_service::finish_export_cancellation(&request.request_id);
        }
        let status = if response["ok"] == true {
            RequestStatus::Completed
        } else if response["error_code"] == "cancelled" {
            RequestStatus::Cancelled
        } else {
            RequestStatus::Failed
        };
        let encoded = encode(response);
        self.remember(request.request_id, fingerprint, encoded.clone(), status);
        encoded
    }

    fn execute(&self, request: &Request, workspace: &DocumentWorkspace) -> Value {
        match request.command {
            Command::GetInstanceState => self.instance_state(request, workspace),
            Command::ListDocumentState => self.list_documents(request, workspace),
            Command::GetDocumentState => self.document_state(request, workspace),
            Command::GetRequestStatus => self.request_status(request),
            Command::CancelRequest => self.cancel_request(request),
            Command::ExportPng => self.export(request, workspace, ExportFormat::Png),
            Command::ExportPdf => self.export(request, workspace, ExportFormat::Pdf),
            Command::ExportHtml => self.export(request, workspace, ExportFormat::Html),
            command if !command.is_read_only() => error(
                &request.request_id,
                command.name(),
                "command_not_ready",
                "The command is allowlisted but its production service is not connected yet.",
            ),
            _ => error(
                &request.request_id,
                request.command.name(),
                "invalid_request",
                "Unsupported request shape.",
            ),
        }
    }

    fn instance_state(&self, request: &Request, workspace: &DocumentWorkspace) -> Value {
        success(
            request,
            json!({
                "version": APP_VERSION,
                "build": APP_VERSION,
                "platform": APP_BUILD_PLATFORM,
                "arch": APP_BUILD_TARGET,
                "pid": self.identity.pid(),
                "instance_id": self.identity.instance_id(),
                "protocol_version": self.identity.protocol_version(),
                "socket_path": self.identity.socket_path(),
                "document_count": workspace.document_ids().len(),
            }),
        )
    }

    fn list_documents(&self, request: &Request, workspace: &DocumentWorkspace) -> Value {
        let documents = workspace
            .document_ids()
            .into_iter()
            .filter_map(|id| workspace.document_by_id(id))
            .map(|document| document_value(document, workspace.active_document_id()))
            .collect::<Vec<_>>();
        success(request, json!({ "documents": documents }))
    }

    fn document_state(&self, request: &Request, workspace: &DocumentWorkspace) -> Value {
        let Some(document_id) = request.target.document_id else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_document",
                "A document id is required.",
            );
        };
        let Some(expected_version) = request.target.expected_version else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_document_version",
                "An expected document version is required.",
            );
        };
        let Some(document) = workspace.document_by_id(document_id) else {
            return error(
                &request.request_id,
                request.command.name(),
                "document_not_found",
                "The document is not open.",
            );
        };
        if document.version() != expected_version {
            return error(
                &request.request_id,
                request.command.name(),
                "stale_document_version",
                "The expected document version is stale.",
            );
        }
        success(
            request,
            json!({ "document": document_value(document, workspace.active_document_id()) }),
        )
    }

    fn request_status(&self, request: &Request) -> Value {
        let Some(reference) = &request.request else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_request_reference",
                "A target request id is required.",
            );
        };
        let Some(cached) = self.cache.get(&reference.request_id) else {
            return error(
                &request.request_id,
                request.command.name(),
                "request_not_found",
                "The request is not in the process-lifetime cache.",
            );
        };
        success(
            request,
            json!({ "target_request_id": reference.request_id, "status": cached.status.name() }),
        )
    }

    fn export(
        &self,
        request: &Request,
        workspace: &DocumentWorkspace,
        format: ExportFormat,
    ) -> Value {
        let Some(document_id) = request.target.document_id else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_document",
                "A document id is required.",
            );
        };
        let Some(expected_version) = request.target.expected_version else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_document_version",
                "An expected document version is required.",
            );
        };
        let Some(output) = &request.output else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_output",
                "An explicit output target is required.",
            );
        };
        let Some(document) = workspace.document_by_id(document_id) else {
            return error(
                &request.request_id,
                request.command.name(),
                "document_not_found",
                "The document is not open.",
            );
        };
        if document.version() != expected_version {
            return error(
                &request.request_id,
                request.command.name(),
                "stale_document_version",
                "The expected document version is stale.",
            );
        }
        let cancellation = export_service::begin_export_cancellation(&request.request_id);
        let result = export_service::export_document_to_path(
            document,
            format,
            Path::new(&output.path),
            output.overwrite,
            &cancellation,
        );
        match result {
            Ok(result) => success(
                request,
                json!({
                    "instance_id": self.identity.instance_id(),
                    "document_id": document_id,
                    "document_version": expected_version,
                    "path": result.path,
                    "sha256": result.sha256,
                    "bytes": result.bytes,
                    "width": result.width,
                    "height": result.height,
                    "page_count": result.page_count,
                }),
            ),
            Err(failure) => error(
                &request.request_id,
                request.command.name(),
                failure.code,
                &failure.message,
            ),
        }
    }

    fn cancel_request(&self, request: &Request) -> Value {
        let Some(reference) = &request.request else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_request_reference",
                "A target request id is required.",
            );
        };
        let Some(cached) = self.cache.get(&reference.request_id) else {
            return error(
                &request.request_id,
                request.command.name(),
                "request_not_found",
                "The request is not in the process-lifetime cache.",
            );
        };
        let status = match cached.status {
            RequestStatus::Queued | RequestStatus::Running | RequestStatus::Cancelled => {
                RequestStatus::Cancelled
            }
            _ => RequestStatus::TooLate,
        };
        success(
            request,
            json!({ "target_request_id": reference.request_id, "status": status.name() }),
        )
    }

    fn remember(
        &mut self,
        id: String,
        fingerprint: String,
        response: Vec<u8>,
        status: RequestStatus,
    ) {
        while self.order.len() >= CACHE_LIMIT {
            if let Some(expired) = self.order.pop_front() {
                self.cache.remove(&expired);
            }
        }
        self.order.push_back(id.clone());
        self.cache.insert(
            id,
            CachedRequest {
                fingerprint,
                response,
                status,
            },
        );
    }
}
