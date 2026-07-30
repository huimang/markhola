use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

use serde::Serialize;
use tao::window::WindowId;

use crate::app::{AppLanguage, ThemePreference};
use crate::document::{DocumentSnapshot, DocumentTabSnapshot};

#[derive(Clone, Debug)]
pub(crate) enum UserEvent {
    NewDocument,
    OpenFile(ActionContext),
    OpenPath(OpenPathRequest),
    ActivateDocument(u64),
    ActivateDocumentAtIndex(usize),
    ActivateNextDocument,
    ActivatePreviousDocument,
    CloseDocument(u64),
    CloseCurrentDocument,
    CloseOtherDocuments,
    CloseAllDocuments,
    ShellReady(WindowId),
    RecoverShell(WindowId, String),
    OpenExternal(String),
    SaveDocument,
    SaveDocumentAs,
    ExportPdf,
    ExportHtml,
    PrintDocument,
    OpenFind,
    ToggleMode,
    SelectTheme(ThemePreference),
    SelectLanguage(AppLanguage),
    IncreaseDocumentSize,
    DecreaseDocumentSize,
    ResetDocumentSize,
    ToggleOutline,
    ToggleFullscreen,
    EditorChanged(WindowId, String),
    ShowAbout,
    OpenDocumentation,
    ProtocolRequest(ProtocolRequestEnvelope),
    Exit,
}

#[derive(Clone, Debug)]
pub(crate) struct ProtocolRequestEnvelope {
    pub(crate) payload: Vec<u8>,
    pub(crate) response: SyncSender<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(crate) enum PendingChangesAction {
    Save,
    Discard,
    Cancel,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ErrorStatusPayload<'a> {
    pub(crate) message: &'a str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ExportSuccessPayload<'a> {
    pub(crate) message: &'a str,
    pub(crate) output_path: String,
    pub(crate) action_label: &'a str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WorkspacePresentation {
    pub(crate) tabs: Vec<DocumentTabSnapshot>,
    pub(crate) active_document: Option<DocumentSnapshot>,
    pub(crate) status_message: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ActionContext {
    pub(crate) event_id: u64,
    pub(crate) source: &'static str,
}

#[derive(Clone, Debug)]
pub(crate) struct OpenPathRequest {
    pub(crate) ctx: ActionContext,
    pub(crate) path: PathBuf,
}
