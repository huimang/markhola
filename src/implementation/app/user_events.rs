use tao::event_loop::{ControlFlow, EventLoopWindowTarget};
use tao::window::Fullscreen;

use crate::app::{AppLanguage, set_current_language, text};

use super::close_actions::close_document_model;
use super::document_actions::{
    create_blank_document_model, open_document_model, open_documents_dialog,
};
use super::export_actions;
use super::navigation_actions::{
    activate_document, activate_document_at_index, close_all_documents, close_current_document,
    close_other_documents, exit_application, switch_document,
};
use super::runtime::AppRuntime;
use super::save_actions::{save_active_document, save_active_document_as};
use super::shell_events::{handle_shell_ready, open_documentation, recover_shell};
use super::surface_actions::present_document_in_surface;
use super::theme_preferences;
use super::workspace_view::{
    present_workspace, render_about, render_error_status, sync_native_language_state,
    sync_native_theme_state,
};
use super::{OpenPathRequest, UserEvent, log_event};
use crate::app::WebStrings;

pub(super) fn handle_user_event(
    user_event: UserEvent,
    target: &EventLoopWindowTarget<UserEvent>,
    runtime: &mut AppRuntime,
    control_flow: &mut ControlFlow,
) {
    match user_event {
        UserEvent::NewDocument => {
            let document_id = create_blank_document_model(&mut runtime.workspace);
            if !present_document_in_surface(
                target,
                runtime,
                document_id,
                text("status.new_document"),
            ) {
                runtime.workspace.close_document(document_id);
            }
        }
        UserEvent::OpenFile(ctx) => handle_open_file(ctx, target, runtime),
        UserEvent::OpenPath(request) => handle_open_path(request, target, runtime),
        UserEvent::ActivateDocument(document_id) => activate_document(document_id, runtime),
        UserEvent::ActivateDocumentAtIndex(index) => activate_document_at_index(index, runtime),
        UserEvent::ActivateNextDocument => switch_document(runtime, true),
        UserEvent::ActivatePreviousDocument => switch_document(runtime, false),
        UserEvent::CloseDocument(document_id) => {
            if close_document_model(
                &runtime.window,
                &runtime.webview,
                &mut runtime.workspace,
                document_id,
                &runtime.asset_access,
            ) {
                super::surface_actions::remove_closed_surface(
                    runtime,
                    document_id,
                    text("status.document_closed"),
                );
            }
        }
        UserEvent::CloseCurrentDocument => close_current_document(runtime, control_flow),
        UserEvent::CloseOtherDocuments => close_other_documents(runtime),
        UserEvent::CloseAllDocuments => close_all_documents(runtime),
        UserEvent::ShellReady(window_id) => {
            activate_event_surface(window_id, runtime);
            handle_shell_ready(runtime);
        }
        UserEvent::RecoverShell(window_id, url) => {
            activate_event_surface(window_id, runtime);
            recover_shell(url, runtime);
        }
        UserEvent::OpenExternal(href) => open_external_link(&href, runtime),
        UserEvent::SaveDocument => {
            save_active_document(
                &runtime.window,
                &runtime.webview,
                &runtime.native_footer,
                &mut runtime.workspace,
                &runtime.asset_access,
            );
        }
        UserEvent::SaveDocumentAs => {
            save_active_document_as(
                &runtime.window,
                &runtime.webview,
                &runtime.native_footer,
                &mut runtime.workspace,
                &runtime.asset_access,
            );
        }
        UserEvent::ExportPng => {
            let context = crate::pdf_export::RenderContext::new(runtime.document_size);
            export_actions::export_png(
                &runtime.webview,
                &runtime.workspace,
                runtime.selected_theme,
                context,
            )
        }
        UserEvent::ExportPdf => {
            let context = crate::pdf_export::RenderContext::new(runtime.document_size);
            export_actions::export_pdf(
                &runtime.webview,
                &runtime.workspace,
                runtime.selected_theme,
                context,
            )
        }
        UserEvent::ExportHtml => {
            let context = crate::pdf_export::RenderContext::new(runtime.document_size);
            export_actions::export_html(
                &runtime.webview,
                &runtime.workspace,
                runtime.selected_theme,
                context,
            )
        }
        UserEvent::PrintDocument => {
            let context = crate::pdf_export::RenderContext::new(runtime.document_size);
            export_actions::print_document(&runtime.webview, &runtime.workspace, context)
        }
        UserEvent::OpenFind => {
            export_actions::open_find_panel(&runtime.webview, &runtime.workspace)
        }
        UserEvent::ToggleMode => toggle_mode(runtime),
        UserEvent::SelectTheme(preference) => {
            super::theme_actions::select_theme(preference, runtime)
        }
        UserEvent::SelectLanguage(language) => select_language(language, runtime),
        UserEvent::IncreaseDocumentSize => {
            update_document_size(runtime.document_size.increase(), runtime)
        }
        UserEvent::DecreaseDocumentSize => {
            update_document_size(runtime.document_size.decrease(), runtime)
        }
        UserEvent::ResetDocumentSize => {
            update_document_size(crate::app::DocumentSize::default(), runtime)
        }
        UserEvent::ToggleOutline => toggle_outline(runtime),
        UserEvent::ToggleFullscreen => toggle_fullscreen(runtime),
        UserEvent::EditorChanged(window_id, markdown) => {
            activate_event_surface(window_id, runtime);
            editor_changed(markdown, runtime);
        }
        UserEvent::ShowAbout => render_about(&runtime.webview),
        UserEvent::OpenDocumentation => open_documentation(runtime),
        UserEvent::ProtocolRequest(request) => {
            let response = runtime.protocol_commands.handle_app_mut_with_theme(
                &request.payload,
                &mut runtime.workspace,
                &runtime.asset_access,
                runtime.selected_theme,
            );
            if protocol_request_changes_active_document(
                &request.payload,
                &response,
                &runtime.workspace,
            ) {
                super::surface_actions::sync_active_surface(runtime, "", true);
            }
            let _ = request.response.send(response);
        }
        UserEvent::Exit => exit_application(runtime, control_flow),
    }
}

fn protocol_request_changes_active_document(
    payload: &[u8],
    response: &[u8],
    workspace: &crate::workspace::DocumentWorkspace,
) -> bool {
    let response_ok = serde_json::from_slice::<serde_json::Value>(response)
        .ok()
        .and_then(|value| value.get("ok").and_then(serde_json::Value::as_bool))
        .unwrap_or(false);
    if !response_ok {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(payload) else {
        return false;
    };
    matches!(
        value.get("command").and_then(serde_json::Value::as_str),
        Some("replace_document_content" | "set_document_mode")
    ) && value
        .pointer("/target/document_id")
        .and_then(serde_json::Value::as_u64)
        == workspace.active_document_id()
}

fn activate_event_surface(window_id: tao::window::WindowId, runtime: &mut AppRuntime) {
    if runtime.active_window_id() == window_id {
        return;
    }
    super::surface_actions::activate_window_surface(
        window_id,
        runtime,
        text("status.document_switched"),
    );
}

fn select_language(language: AppLanguage, runtime: &mut AppRuntime) {
    if runtime.language == language {
        return;
    }
    #[cfg(target_os = "macos")]
    let outline_selected = super::macos_menu::outline_selected();

    runtime.language = language;
    set_current_language(language);
    theme_preferences::save_app_language(language);

    #[cfg(target_os = "macos")]
    {
        if let Err(error) = super::macos_menu::install(&runtime.proxy) {
            let message = text("status.failed_rebuild_menu").replace("{error}", &error.to_string());
            render_error_status(&runtime.webview, &message);
            return;
        }
        sync_native_menu_state_after_language_change(runtime);
        super::macos_menu::set_outline_selected(outline_selected);
    }

    let strings = WebStrings::for_language(language);
    if let Ok(payload) = serde_json::to_string(&strings) {
        let _ = runtime
            .webview
            .evaluate_script(&format!("window.applyAppLanguage({payload});"));
        for surface in runtime.inactive_surfaces.values() {
            let _ = surface
                .webview
                .evaluate_script(&format!("window.applyAppLanguage({payload});"));
        }
    }
    let ready = text("status.ready");
    runtime.native_footer.sync(&runtime.workspace, ready);
    for surface in runtime.inactive_surfaces.values() {
        let snapshot = surface
            .document_id
            .and_then(|document_id| runtime.workspace.document_snapshot(document_id));
        surface.native_footer.sync_document(snapshot, ready);
    }
}

#[cfg(target_os = "macos")]
fn sync_native_menu_state_after_language_change(runtime: &AppRuntime) {
    super::workspace_view::sync_native_menu_state(&runtime.workspace);
    sync_native_theme_state(runtime.theme_preference);
    sync_native_language_state(runtime.language);
}

fn handle_open_file(
    ctx: super::ActionContext,
    target: &EventLoopWindowTarget<UserEvent>,
    runtime: &mut AppRuntime,
) {
    log_event(
        "user_event.received",
        Some(ctx.event_id),
        "handling UserEvent::OpenFile",
        format!("source={}", ctx.source),
    );
    match open_documents_dialog(ctx.event_id) {
        Some(paths) => {
            let mut failures = 0usize;
            for path in paths {
                match open_document_model(
                    &mut runtime.workspace,
                    &path,
                    Some(ctx.event_id),
                    &runtime.asset_access,
                ) {
                    Ok(result) => {
                        let document_id = match result {
                            crate::workspace::WorkspaceOpenResult::OpenedNew(document_id)
                            | crate::workspace::WorkspaceOpenResult::ActivatedExisting(
                                document_id,
                            ) => document_id,
                        };
                        let status = if matches!(
                            result,
                            crate::workspace::WorkspaceOpenResult::ActivatedExisting(_)
                        ) {
                            text("status.already_open")
                        } else {
                            text("status.document_loaded")
                        };
                        if !present_document_in_surface(target, runtime, document_id, status) {
                            if matches!(result, crate::workspace::WorkspaceOpenResult::OpenedNew(_))
                            {
                                runtime.workspace.close_document(document_id);
                                super::asset_access::unregister_document(
                                    &runtime.asset_access,
                                    document_id,
                                );
                            }
                            failures += 1;
                        }
                    }
                    Err(message) => {
                        render_error_status(&runtime.webview, &message);
                        failures += 1;
                    }
                }
            }
            if failures > 0 {
                render_error_status(&runtime.webview, text("status.some_open_failed"));
            }
        }
        None => {}
    }
}

fn handle_open_path(
    request: OpenPathRequest,
    target: &EventLoopWindowTarget<UserEvent>,
    runtime: &mut AppRuntime,
) {
    let OpenPathRequest { ctx, path } = request;
    log_event(
        "user_event.received",
        Some(ctx.event_id),
        "handling UserEvent::OpenPath",
        format!("source={} path={}", ctx.source, path.display()),
    );
    if !runtime.shell.ready {
        runtime
            .shell
            .pending_open_requests
            .push(OpenPathRequest { ctx, path });
        return;
    }
    match open_document_model(
        &mut runtime.workspace,
        &path,
        Some(ctx.event_id),
        &runtime.asset_access,
    ) {
        Ok(result) => {
            let document_id = match result {
                crate::workspace::WorkspaceOpenResult::OpenedNew(document_id)
                | crate::workspace::WorkspaceOpenResult::ActivatedExisting(document_id) => {
                    document_id
                }
            };
            let status = if matches!(
                result,
                crate::workspace::WorkspaceOpenResult::ActivatedExisting(_)
            ) {
                text("status.already_open")
            } else {
                text("status.document_loaded")
            };
            if !present_document_in_surface(target, runtime, document_id, status)
                && matches!(result, crate::workspace::WorkspaceOpenResult::OpenedNew(_))
            {
                runtime.workspace.close_document(document_id);
                super::asset_access::unregister_document(&runtime.asset_access, document_id);
            }
        }
        Err(message) => render_error_status(&runtime.webview, &message),
    }
}

fn toggle_mode(runtime: &mut AppRuntime) {
    let toggled = runtime.workspace.active_document_mut().map(|document| {
        document.toggle_mode();
    });
    match toggled {
        Some(()) => present_workspace(
            &runtime.window,
            &runtime.webview,
            &runtime.native_footer,
            &runtime.workspace,
            text("status.ready"),
            true,
        ),
        None => render_error_status(&runtime.webview, text("status.no_document")),
    }
}

fn update_document_size(size: crate::app::DocumentSize, runtime: &mut AppRuntime) {
    runtime.document_size = size;
    theme_preferences::save_document_size(size);
    let script = format!("window.applyDocumentSize({});", size.percent());
    if let Err(error) = runtime.webview.evaluate_script(&script) {
        let message = text("status.failed_apply_size").replace("{error}", &error.to_string());
        render_error_status(&runtime.webview, &message);
        return;
    }
    for surface in runtime.inactive_surfaces.values() {
        let _ = surface.webview.evaluate_script(&script);
    }
}

fn toggle_outline(runtime: &mut AppRuntime) {
    if runtime
        .workspace
        .active_document()
        .is_some_and(|document| document.mode() == crate::document::DocumentMode::Readonly)
    {
        #[cfg(target_os = "macos")]
        let selected = super::macos_menu::toggle_outline_selected();
        #[cfg(not(target_os = "macos"))]
        let selected = false;
        let script = format!("window.setOutlinePanelOpen({selected});");
        if let Err(error) = runtime.webview.evaluate_script(&script) {
            let status =
                text("status.failed_toggle_outline").replace("{error}", &error.to_string());
            render_error_status(&runtime.webview, &status);
        }
    }
}

fn toggle_fullscreen(runtime: &mut AppRuntime) {
    let next_state = if runtime.window.fullscreen().is_some() {
        None
    } else {
        Some(Fullscreen::Borderless(None))
    };
    runtime.window.set_fullscreen(next_state);
}

fn editor_changed(markdown: String, runtime: &mut AppRuntime) {
    if let Some(document) = runtime.workspace.active_document_mut() {
        document.update_markdown(markdown);
        super::surface_actions::sync_active_surface(runtime, text("status.unsaved"), false);
    }
}

fn open_external_link(href: &str, runtime: &AppRuntime) {
    if let Err(error) = open::that(href) {
        log_event(
            "open_external.error",
            None,
            "open external failed",
            format!("error={error}"),
        );
        let message = text("status.failed_open_link").replace("{error}", &error.to_string());
        render_error_status(&runtime.webview, &message);
    }
}
