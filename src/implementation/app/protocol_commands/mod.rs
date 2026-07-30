mod response;
mod schema;

#[cfg(test)]
#[path = "control_tests.rs"]
mod control_tests;
#[cfg(test)]
#[path = "save_tests.rs"]
mod save_tests;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, VecDeque};
use std::path::Path;

use serde_json::{Value, json};

use crate::app::{APP_BUILD_PLATFORM, APP_BUILD_TARGET, APP_VERSION};
use crate::document::DocumentMode;
use crate::export_service::{self, ExportFormat};
use crate::file_io;
use crate::save_service;
use crate::workspace::{DocumentWorkspace, WorkspaceOpenResult};

use super::asset_access::AssetAccessRegistry;
use super::document_actions::open_document_model;
use super::protocol_transport::ProtocolIdentity;
use response::{document_value, encode, error, sha256, success};
use schema::{Command, DocumentModeRequest, Request};

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
        let status = if response["ok"] == true {
            RequestStatus::Completed
        } else if response["error_code"] == "cancelled" {
            RequestStatus::Cancelled
        } else {
            RequestStatus::Failed
        };
        if request.command.is_export() {
            let export_status = match status {
                RequestStatus::Completed => export_service::ExportStatus::Completed,
                RequestStatus::Cancelled => export_service::ExportStatus::Cancelled,
                _ => export_service::ExportStatus::Failed,
            };
            export_service::finish_export(&request.request_id, export_status);
        }
        let encoded = encode(response);
        self.remember(request.request_id, fingerprint, encoded.clone(), status);
        encoded
    }

    pub(super) fn handle_mut(
        &mut self,
        payload: &[u8],
        workspace: &mut DocumentWorkspace,
    ) -> Vec<u8> {
        self.handle_mut_with_assets(payload, workspace, None)
    }

    pub(super) fn handle_app_mut(
        &mut self,
        payload: &[u8],
        workspace: &mut DocumentWorkspace,
        asset_access: &AssetAccessRegistry,
    ) -> Vec<u8> {
        self.handle_mut_with_assets(payload, workspace, Some(asset_access))
    }

    fn handle_mut_with_assets(
        &mut self,
        payload: &[u8],
        workspace: &mut DocumentWorkspace,
        asset_access: Option<&AssetAccessRegistry>,
    ) -> Vec<u8> {
        let Ok(request) = serde_json::from_slice::<Request>(payload) else {
            return self.handle(payload, workspace);
        };
        if !matches!(
            request.command,
            Command::SaveDocument
                | Command::SaveDocumentAs
                | Command::OpenDocument
                | Command::ReplaceDocumentContent
                | Command::SetDocumentMode
                | Command::WaitRenderReady
        ) {
            return self.handle(payload, workspace);
        }

        let fingerprint = sha256(payload);
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
            match request.command {
                Command::SaveDocument | Command::SaveDocumentAs => self.save(&request, workspace),
                _ => self.control(&request, workspace, asset_access),
            }
        };
        let status = if response["ok"] == true {
            RequestStatus::Completed
        } else {
            RequestStatus::Failed
        };
        let encoded = encode(response);
        self.remember(request.request_id, fingerprint, encoded.clone(), status);
        encoded
    }

    fn control(
        &self,
        request: &Request,
        workspace: &mut DocumentWorkspace,
        asset_access: Option<&AssetAccessRegistry>,
    ) -> Value {
        match request.command {
            Command::OpenDocument => self.open_document(request, workspace, asset_access),
            Command::ReplaceDocumentContent => self.replace_document_content(request, workspace),
            Command::SetDocumentMode => self.set_document_mode(request, workspace),
            Command::WaitRenderReady => self.wait_render_ready(request, workspace),
            _ => unreachable!("control is called only for control-plane commands"),
        }
    }

    fn open_document(
        &self,
        request: &Request,
        workspace: &mut DocumentWorkspace,
        asset_access: Option<&AssetAccessRegistry>,
    ) -> Value {
        let Some(requested_path) = request.path.as_deref() else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_path",
                "An explicit document path is required.",
            );
        };
        let path = Path::new(requested_path);
        if !path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )
            })
        {
            return error(
                &request.request_id,
                request.command.name(),
                "invalid_document_path",
                "The document path must be absolute and normalized.",
            );
        }
        if let Err(message) = file_io::ensure_supported_markdown_extension(path) {
            return error(
                &request.request_id,
                request.command.name(),
                "invalid_document_extension",
                &message,
            );
        }
        let Ok(metadata) = std::fs::symlink_metadata(path) else {
            return error(
                &request.request_id,
                request.command.name(),
                "document_unavailable",
                "The document path is unavailable.",
            );
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return error(
                &request.request_id,
                request.command.name(),
                "unsafe_document_path",
                "The document path must be a regular file.",
            );
        }
        let Ok(canonical) = path.canonicalize() else {
            return error(
                &request.request_id,
                request.command.name(),
                "document_unavailable",
                "The document path cannot be canonicalized.",
            );
        };
        let Some(asset_access) = asset_access else {
            return error(
                &request.request_id,
                request.command.name(),
                "command_not_ready",
                "The production document service is unavailable.",
            );
        };
        let previous_active = workspace.active_document_id();
        let result = open_document_model(workspace, &canonical, None, asset_access);
        let document_id = match result {
            Ok(WorkspaceOpenResult::OpenedNew(id) | WorkspaceOpenResult::ActivatedExisting(id)) => {
                id
            }
            Err(message) => {
                return error(
                    &request.request_id,
                    request.command.name(),
                    "document_open_failed",
                    &message,
                );
            }
        };
        if let Some(previous_active) = previous_active {
            workspace.activate_document(previous_active);
        }
        let document = workspace
            .document_by_id(document_id)
            .expect("production open service returned an open document");
        success(
            request,
            json!({
                "instance_id": self.identity.instance_id(),
                "document": document_value(document, workspace.active_document_id()),
            }),
        )
    }

    fn replace_document_content(
        &self,
        request: &Request,
        workspace: &mut DocumentWorkspace,
    ) -> Value {
        let Some(document_id) = request.target.document_id else {
            return missing_document(request);
        };
        let Some(expected_version) = request.target.expected_version else {
            return missing_version(request);
        };
        let Some(content) = request.content.as_ref() else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_content",
                "Full replacement content is required.",
            );
        };
        let Some(document) = workspace.document_by_id(document_id) else {
            return document_not_found(request);
        };
        if document.version() != expected_version {
            return stale_version(request);
        }
        if document.mode() != DocumentMode::Writable {
            return error(
                &request.request_id,
                request.command.name(),
                "document_readonly",
                "The document must be writable before replacing content.",
            );
        }
        if document.is_dirty() {
            return error(
                &request.request_id,
                request.command.name(),
                "document_dirty",
                "Unsaved document content cannot be replaced.",
            );
        }
        let source_changed = !document.is_draft()
            && match file_io::load_markdown(document.canonical_path()) {
                Ok(disk) => disk != document.saved_markdown(),
                Err(_) => true,
            };
        if source_changed {
            return error(
                &request.request_id,
                request.command.name(),
                "external_source_changed",
                "The source file changed outside MarkHola.",
            );
        }
        let active_document_id = workspace.active_document_id();
        let document = workspace
            .document_by_id_mut(document_id)
            .expect("validated document remains open");
        document.update_markdown(content.clone());
        success(
            request,
            json!({
                "instance_id": self.identity.instance_id(),
                "document": document_value(document, active_document_id),
            }),
        )
    }

    fn set_document_mode(&self, request: &Request, workspace: &mut DocumentWorkspace) -> Value {
        let Some(document_id) = request.target.document_id else {
            return missing_document(request);
        };
        let Some(expected_version) = request.target.expected_version else {
            return missing_version(request);
        };
        let Some(mode) = request.mode else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_document_mode",
                "An explicit readonly or edit mode is required.",
            );
        };
        let active_document_id = workspace.active_document_id();
        let Some(document) = workspace.document_by_id_mut(document_id) else {
            return document_not_found(request);
        };
        if document.version() != expected_version {
            return stale_version(request);
        }
        document.set_mode(match mode {
            DocumentModeRequest::Readonly => DocumentMode::Readonly,
            DocumentModeRequest::Edit => DocumentMode::Writable,
        });
        success(
            request,
            json!({
                "instance_id": self.identity.instance_id(),
                "document": document_value(document, active_document_id),
            }),
        )
    }

    fn wait_render_ready(&self, request: &Request, workspace: &DocumentWorkspace) -> Value {
        let Some(document_id) = request.target.document_id else {
            return missing_document(request);
        };
        let Some(expected_version) = request.target.expected_version else {
            return missing_version(request);
        };
        let Some(expected_generation) = request.render_generation else {
            return error(
                &request.request_id,
                request.command.name(),
                "missing_render_generation",
                "An exact render generation is required.",
            );
        };
        let Some(document) = workspace.document_by_id(document_id) else {
            return document_not_found(request);
        };
        if document.version() != expected_version {
            return stale_version(request);
        }
        if document.render_generation() != expected_generation {
            return error(
                &request.request_id,
                request.command.name(),
                "render_generation_mismatch",
                "The exact document render generation is not ready.",
            );
        }
        success(
            request,
            json!({
                "instance_id": self.identity.instance_id(),
                "document_id": document_id,
                "document_version": expected_version,
                "render_generation": expected_generation,
                "status": "ready",
                "render_errors": [],
            }),
        )
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

    fn save(&self, request: &Request, workspace: &mut DocumentWorkspace) -> Value {
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

        let result = match request.command {
            Command::SaveDocument => {
                if let Some(output) = &request.output {
                    let requested = Path::new(&output.path);
                    let matches_source = requested
                        .canonicalize()
                        .is_ok_and(|path| path == document.canonical_path());
                    if !matches_source {
                        return error(
                            &request.request_id,
                            request.command.name(),
                            "save_target_mismatch",
                            "save_document must use the existing document path.",
                        );
                    }
                }
                let document = workspace
                    .document_by_id_mut(document_id)
                    .expect("validated document remains open");
                save_service::save_document(document)
            }
            Command::SaveDocumentAs => {
                let Some(output) = &request.output else {
                    return error(
                        &request.request_id,
                        request.command.name(),
                        "missing_output",
                        "An explicit Save As target is required.",
                    );
                };
                let target = match save_service::validate_save_as_target(
                    Path::new(&output.path),
                    output.overwrite,
                ) {
                    Ok(target) => target,
                    Err(failure) => {
                        return error(
                            &request.request_id,
                            request.command.name(),
                            failure.code,
                            &failure.message,
                        );
                    }
                };
                if workspace
                    .find_by_path_excluding(&target, document_id)
                    .is_some()
                {
                    return error(
                        &request.request_id,
                        request.command.name(),
                        "save_target_open",
                        "The Save As target is already open.",
                    );
                }
                let document = workspace
                    .document_by_id_mut(document_id)
                    .expect("validated document remains open");
                save_service::save_document_as(document, &target, output.overwrite)
            }
            _ => unreachable!("save is called only for save commands"),
        };
        let path = match result {
            Ok(path) => path,
            Err(failure) => {
                return error(
                    &request.request_id,
                    request.command.name(),
                    failure.code,
                    &failure.message,
                );
            }
        };
        let document = workspace
            .document_by_id(document_id)
            .expect("saved document remains open");
        success(
            request,
            json!({
                "instance_id": self.identity.instance_id(),
                "document_id": document.id(),
                "document_version": document.version(),
                "path": path,
                "content_sha256": document.content_sha256(),
                "bytes": document.markdown().len(),
            }),
        )
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

fn missing_document(request: &Request) -> Value {
    error(
        &request.request_id,
        request.command.name(),
        "missing_document",
        "A document id is required.",
    )
}

fn missing_version(request: &Request) -> Value {
    error(
        &request.request_id,
        request.command.name(),
        "missing_document_version",
        "An expected document version is required.",
    )
}

fn document_not_found(request: &Request) -> Value {
    error(
        &request.request_id,
        request.command.name(),
        "document_not_found",
        "The document is not open.",
    )
}

fn stale_version(request: &Request) -> Value {
    error(
        &request.request_id,
        request.command.name(),
        "stale_document_version",
        "The expected document version is stale.",
    )
}
