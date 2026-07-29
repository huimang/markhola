use crate::app::text;
use tao::event_loop::ControlFlow;

use super::close_actions::{
    close_document_model, close_document_models, resolve_all_pending_changes,
};
use super::native_tabs;
use super::runtime::AppRuntime;
use super::surface_actions::activate_document_surface;
use super::surface_actions::{remove_closed_surface, reset_to_empty, sync_active_surface};
use super::workspace_view::render_error_status;

pub(super) fn activate_document(document_id: u64, runtime: &mut AppRuntime) {
    if !activate_document_surface(document_id, runtime, text("status.document_switched")) {
        render_error_status(&runtime.webview, text("status.missing_tab"));
    }
}

pub(super) fn activate_document_at_index(index: usize, runtime: &mut AppRuntime) {
    let Some(document_id) = native_tabs::visible_document_ids(runtime)
        .get(index)
        .copied()
    else {
        return;
    };
    activate_document(document_id, runtime);
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
        if let Some(document_id) = runtime.workspace.active_document_id() {
            activate_document_surface(document_id, runtime, message);
        }
    }
}

pub(super) fn close_current_document(runtime: &mut AppRuntime, control_flow: &mut ControlFlow) {
    if let Some(document_id) = runtime.workspace.active_document_id() {
        if close_document_model(
            &runtime.window,
            &runtime.webview,
            &mut runtime.workspace,
            document_id,
            &runtime.asset_access,
        ) {
            remove_closed_surface(runtime, document_id, text("status.document_closed"));
        }
    } else {
        *control_flow = ControlFlow::Exit;
    }
}

pub(super) fn close_other_documents(runtime: &mut AppRuntime) {
    if let Some(active_document_id) = runtime.workspace.active_document_id() {
        let document_ids = runtime.workspace.other_document_ids(active_document_id);
        if !document_ids.is_empty() {
            if close_document_models(
                &runtime.window,
                &runtime.webview,
                &mut runtime.workspace,
                &document_ids,
                &runtime.asset_access,
            ) {
                for document_id in document_ids {
                    let _ = runtime.remove_inactive_surface_for_document(document_id);
                }
                sync_active_surface(runtime, text("status.other_tabs_closed"), false);
            }
        }
    }
}

pub(super) fn close_all_documents(runtime: &mut AppRuntime) {
    let document_ids = runtime.workspace.document_ids();
    if !document_ids.is_empty() {
        if close_document_models(
            &runtime.window,
            &runtime.webview,
            &mut runtime.workspace,
            &document_ids,
            &runtime.asset_access,
        ) {
            reset_to_empty(runtime, text("status.all_tabs_closed"));
        }
    }
}

pub(super) fn exit_application(runtime: &mut AppRuntime, control_flow: &mut ControlFlow) {
    if resolve_all_pending_changes(&runtime.window, &runtime.webview, &mut runtime.workspace) {
        *control_flow = ControlFlow::Exit;
    }
}
