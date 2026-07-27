use tao::event::{ElementState, WindowEvent};
use tao::event_loop::ControlFlow;
use tao::window::WindowId;

use super::runtime::AppRuntime;
use super::shortcuts::handle_command_shortcut;
use super::workspace_view::render_status;
use super::{UserEvent, dispatch_user_event, log_event, new_action_context};
use crate::app::text;

pub(super) fn handle_window_event(
    window_id: WindowId,
    event: WindowEvent,
    runtime: &mut AppRuntime,
    control_flow: &mut ControlFlow,
) {
    if matches!(
        event,
        WindowEvent::Focused(true) | WindowEvent::CloseRequested
    ) {
        super::surface_actions::activate_window_surface(
            window_id,
            runtime,
            text("status.document_switched"),
        );
    }
    match event {
        WindowEvent::CloseRequested => handle_close_requested(runtime, control_flow),
        WindowEvent::ModifiersChanged(next_modifiers) => runtime.modifiers = next_modifiers,
        WindowEvent::KeyboardInput { event, .. } => {
            if event.state == ElementState::Released && runtime.modifiers.super_key() {
                handle_command_shortcut(&runtime.proxy, event.physical_key);
            }
        }
        WindowEvent::HoveredFile(path) => {
            let message =
                text("status.drop_to_open").replace("{path}", &path.display().to_string());
            render_status(&runtime.webview, &message, "info");
        }
        WindowEvent::HoveredFileCancelled => {
            render_status(&runtime.webview, text("status.ready_open_hint"), "info");
        }
        WindowEvent::DroppedFile(path) => {
            let ctx = new_action_context("window-dropped-file");
            log_event(
                "window.dropped_file",
                Some(ctx.event_id),
                "window dropped file",
                format!("path={}", path.display()),
            );
            dispatch_user_event(
                &runtime.proxy,
                "window-drop",
                UserEvent::OpenPath(super::OpenPathRequest { ctx, path }),
            );
        }
        WindowEvent::Resized(_) if runtime.active_window_id() == window_id => {
            runtime
                .native_footer
                .relayout(&runtime.window, &runtime.webview);
        }
        WindowEvent::ThemeChanged(theme) => {
            super::theme_actions::system_theme_changed(theme, runtime);
        }
        _ => {}
    }
}

fn handle_close_requested(runtime: &mut AppRuntime, control_flow: &mut ControlFlow) {
    log_event("window.close_requested", None, "window close requested", "");
    super::navigation_actions::close_current_document(runtime, control_flow);
}
