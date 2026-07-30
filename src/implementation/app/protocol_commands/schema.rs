use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Command {
    GetInstanceState,
    ListDocumentState,
    GetDocumentState,
    OpenDocument,
    ReplaceDocumentContent,
    SetDocumentMode,
    WaitRenderReady,
    GetRequestStatus,
    CancelRequest,
    ExportPng,
    ExportPdf,
    ExportHtml,
    SaveDocument,
    SaveDocumentAs,
}

impl Command {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::GetInstanceState => "get_instance_state",
            Self::ListDocumentState => "list_document_state",
            Self::GetDocumentState => "get_document_state",
            Self::OpenDocument => "open_document",
            Self::ReplaceDocumentContent => "replace_document_content",
            Self::SetDocumentMode => "set_document_mode",
            Self::WaitRenderReady => "wait_render_ready",
            Self::GetRequestStatus => "get_request_status",
            Self::CancelRequest => "cancel_request",
            Self::ExportPng => "export_png",
            Self::ExportPdf => "export_pdf",
            Self::ExportHtml => "export_html",
            Self::SaveDocument => "save_document",
            Self::SaveDocumentAs => "save_document_as",
        }
    }

    pub(super) fn is_read_only(self) -> bool {
        matches!(
            self,
            Self::GetInstanceState
                | Self::ListDocumentState
                | Self::GetDocumentState
                | Self::GetRequestStatus
                | Self::CancelRequest
        )
    }

    pub(super) fn is_export(self) -> bool {
        matches!(self, Self::ExportPng | Self::ExportPdf | Self::ExportHtml)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    pub(super) request_id: String,
    pub(super) instance_token: String,
    pub(super) command: Command,
    pub(super) target: Target,
    #[serde(default)]
    pub(super) request: Option<RequestReference>,
    #[serde(default)]
    pub(super) output: Option<Output>,
    #[serde(default)]
    pub(super) path: Option<String>,
    #[serde(default)]
    pub(super) content: Option<String>,
    #[serde(default)]
    pub(super) mode: Option<DocumentModeRequest>,
    #[serde(default)]
    pub(super) render_generation: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum DocumentModeRequest {
    Readonly,
    Edit,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Output {
    pub(super) path: String,
    #[serde(default)]
    pub(super) overwrite: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Target {
    pub(super) instance_id: String,
    #[serde(default)]
    pub(super) document_id: Option<u64>,
    #[serde(default)]
    pub(super) expected_version: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RequestReference {
    pub(super) request_id: String,
}
