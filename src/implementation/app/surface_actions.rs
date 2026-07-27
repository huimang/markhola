use tao::event_loop::EventLoopWindowTarget;
use tao::window::WindowId;

use crate::app::UserEvent;

use super::bootstrap::build_document_surface;
use super::native_tabs;
use super::runtime::AppRuntime;
use super::workspace_view::{present_workspace, render_status};

pub(super) fn present_document_in_surface(
    target: &EventLoopWindowTarget<UserEvent>,
    runtime: &mut AppRuntime,
    document_id: u64,
    status: &str,
) -> bool {
    if let Some(window_id) = runtime.window_id_for_document(document_id) {
        return activate_window_surface(window_id, runtime, status);
    }

    if runtime.active_document_id.is_none() {
        runtime.active_document_id = Some(document_id);
        runtime.workspace.activate_document(document_id);
        sync_active_surface(runtime, status, true);
        return true;
    }

    let surface = match build_document_surface(
        target,
        &runtime.proxy,
        runtime.asset_access.clone(),
        runtime.selected_theme,
        runtime.language,
        runtime.document_size,
        document_id,
    ) {
        Ok(surface) => surface,
        Err(error) => {
            let message = format!("Failed to create native document tab: {error}");
            render_status(&runtime.webview, &message, "error");
            return false;
        }
    };
    let window_id = surface.window_id();
    native_tabs::add_to_group(&runtime.window, &surface.window);
    runtime.insert_surface(surface);
    runtime.activate_surface(window_id);
    runtime.workspace.activate_document(document_id);
    sync_active_surface(runtime, status, true);
    true
}

pub(super) fn activate_document_surface(
    document_id: u64,
    runtime: &mut AppRuntime,
    status: &str,
) -> bool {
    let Some(window_id) = runtime.window_id_for_document(document_id) else {
        return false;
    };
    native_tabs::select_window_id(runtime, window_id);
    activate_window_surface(window_id, runtime, status)
}

pub(super) fn activate_window_surface(
    window_id: WindowId,
    runtime: &mut AppRuntime,
    status: &str,
) -> bool {
    let Some(document_id) = runtime.document_id_for_window(window_id) else {
        return false;
    };
    if !runtime.activate_surface(window_id) {
        return false;
    }
    if !runtime.workspace.activate_document(document_id) {
        return false;
    }
    sync_active_surface(runtime, status, false);
    true
}

pub(super) fn sync_active_surface(runtime: &AppRuntime, status: &str, full_render: bool) {
    present_workspace(
        &runtime.window,
        &runtime.webview,
        &runtime.native_footer,
        &runtime.workspace,
        status,
        full_render,
    );
    let dirty = runtime
        .workspace
        .active_document()
        .is_some_and(|document| document.is_dirty());
    native_tabs::set_document_edited(&runtime.window, dirty);
}

pub(super) fn remove_closed_surface(runtime: &mut AppRuntime, document_id: u64, status: &str) {
    if runtime.active_document_id == Some(document_id) {
        if let Some(next_document_id) = runtime.workspace.active_document_id() {
            if let Some(next_window_id) = runtime.window_id_for_document(next_document_id) {
                runtime.activate_surface(next_window_id);
                let _ = runtime.remove_inactive_surface_for_document(document_id);
                native_tabs::select(&runtime.window);
                sync_active_surface(runtime, status, true);
                return;
            }
        }
        runtime.active_document_id = None;
        sync_active_surface(runtime, status, true);
    } else {
        let _ = runtime.remove_inactive_surface_for_document(document_id);
        sync_active_surface(runtime, status, false);
    }
}

pub(super) fn reset_to_empty(runtime: &mut AppRuntime, status: &str) {
    runtime.reset_to_empty_surface();
    sync_active_surface(runtime, status, true);
}
