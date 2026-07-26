use crate::app::text;
use tao::event_loop::ControlFlow;

use super::close_actions::{close_document_tab, close_document_tabs, resolve_all_pending_changes};
use super::runtime::AppRuntime;
use super::workspace_view::{render_status, sync_workspace_state};

pub(super) fn activate_document(document_id: u64, runtime: &mut AppRuntime) {
    if runtime.workspace.activate_document(document_id) {
        sync_workspace_state(
            &runtime.window,
            &runtime.webview,
            &runtime.native_footer,
            &runtime.workspace,
            text("status.document_switched"),
        );
    } else {
        render_status(&runtime.webview, text("status.missing_tab"), "error");
    }
}

pub(super) fn switch_document(runtime: &mut AppRuntime, next: bool) {
    let changed = if next {
        runtime.workspace.activate_next_document()
    } else {
        runtime.workspace.activate_previous_document()
    };
    if changed {
        let message = if next {
            text("status.next_tab")
        } else {
            text("status.previous_tab")
        };
        sync_workspace_state(
            &runtime.window,
            &runtime.webview,
            &runtime.native_footer,
            &runtime.workspace,
            message,
        );
    }
}

pub(super) fn close_current_document(runtime: &mut AppRuntime, control_flow: &mut ControlFlow) {
    if let Some(document_id) = runtime.workspace.active_document_id() {
        close_document_tab(
            &runtime.window,
            &runtime.webview,
            &runtime.native_footer,
            &mut runtime.workspace,
            document_id,
            text("status.document_closed"),
            &runtime.asset_access,
        );
    } else {
        *control_flow = ControlFlow::Exit;
    }
}

pub(super) fn close_other_documents(runtime: &mut AppRuntime) {
    if let Some(active_document_id) = runtime.workspace.active_document_id() {
        let document_ids = runtime.workspace.other_document_ids(active_document_id);
        if document_ids.is_empty() {
            render_status(&runtime.webview, text("status.no_other_tabs"), "info");
        } else {
            close_document_tabs(
                &runtime.window,
                &runtime.webview,
                &runtime.native_footer,
                &mut runtime.workspace,
                &document_ids,
                text("status.other_tabs_closed"),
                &runtime.asset_access,
            );
        }
    } else {
        render_status(&runtime.webview, text("status.no_document"), "info");
    }
}

pub(super) fn close_all_documents(runtime: &mut AppRuntime) {
    let document_ids = runtime.workspace.document_ids();
    if document_ids.is_empty() {
        render_status(&runtime.webview, text("status.no_document"), "info");
    } else {
        close_document_tabs(
            &runtime.window,
            &runtime.webview,
            &runtime.native_footer,
            &mut runtime.workspace,
            &document_ids,
            text("status.all_tabs_closed"),
            &runtime.asset_access,
        );
    }
}

pub(super) fn exit_application(runtime: &mut AppRuntime, control_flow: &mut ControlFlow) {
    if resolve_all_pending_changes(&runtime.window, &runtime.webview, &mut runtime.workspace) {
        *control_flow = ControlFlow::Exit;
    }
}
