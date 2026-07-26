use tao::event_loop::ControlFlow;
use tao::window::Fullscreen;

use crate::app::{AppLanguage, AppTheme, set_current_language, text};
use crate::render_assets;

use super::close_actions::close_document_tab;
use super::document_actions::{create_blank_document, open_document, open_documents_dialog};
use super::export_actions;
use super::navigation_actions::{
    activate_document, close_all_documents, close_current_document, close_other_documents,
    exit_application, switch_document,
};
use super::runtime::AppRuntime;
use super::save_actions::{save_active_document, save_active_document_as};
use super::shell_events::{handle_shell_ready, open_documentation, recover_shell};
use super::theme_preferences;
use super::workspace_view::{
    present_workspace, render_about, render_status, sync_native_language_state,
    sync_native_theme_state, sync_workspace_state,
};
use super::{OpenPathRequest, UserEvent, log_event};
use crate::app::WebStrings;

pub(super) fn handle_user_event(
    user_event: UserEvent,
    runtime: &mut AppRuntime,
    control_flow: &mut ControlFlow,
) {
    match user_event {
        UserEvent::NewDocument => {
            create_blank_document(
                &runtime.window,
                &runtime.webview,
                &runtime.native_footer,
                &mut runtime.workspace,
            );
        }
        UserEvent::OpenFile(ctx) => handle_open_file(ctx, runtime),
        UserEvent::OpenPath(request) => handle_open_path(request, runtime),
        UserEvent::ActivateDocument(document_id) => activate_document(document_id, runtime),
        UserEvent::ActivateNextDocument => switch_document(runtime, true),
        UserEvent::ActivatePreviousDocument => switch_document(runtime, false),
        UserEvent::CloseDocument(document_id) => {
            close_document_tab(
                &runtime.window,
                &runtime.webview,
                &runtime.native_footer,
                &mut runtime.workspace,
                document_id,
                text("status.document_closed"),
                &runtime.asset_access,
            );
        }
        UserEvent::CloseCurrentDocument => close_current_document(runtime, control_flow),
        UserEvent::CloseOtherDocuments => close_other_documents(runtime),
        UserEvent::CloseAllDocuments => close_all_documents(runtime),
        UserEvent::ShellReady => handle_shell_ready(runtime),
        UserEvent::RecoverShell(url) => recover_shell(url, runtime),
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
        UserEvent::ExportPdf => export_actions::export_pdf(&runtime.webview, &runtime.workspace),
        UserEvent::ExportHtml => export_actions::export_html(&runtime.webview, &runtime.workspace),
        UserEvent::PrintDocument => {
            export_actions::print_document(&runtime.webview, &runtime.workspace)
        }
        UserEvent::OpenFind => {
            export_actions::open_find_panel(&runtime.webview, &runtime.workspace)
        }
        UserEvent::ToggleMode => toggle_mode(runtime),
        UserEvent::SelectTheme(theme) => select_theme(theme, runtime),
        UserEvent::SelectLanguage(language) => select_language(language, runtime),
        UserEvent::IncreaseDocumentSize => update_document_size(
            runtime.document_size.increase(),
            text("status.size_increased_action"),
            runtime,
        ),
        UserEvent::DecreaseDocumentSize => update_document_size(
            runtime.document_size.decrease(),
            text("status.size_decreased_action"),
            runtime,
        ),
        UserEvent::ResetDocumentSize => update_document_size(
            crate::app::DocumentSize::default(),
            text("status.size_reset_action"),
            runtime,
        ),
        UserEvent::ToggleOutline => toggle_outline(runtime),
        UserEvent::ToggleFullscreen => toggle_fullscreen(runtime),
        UserEvent::EditorChanged(markdown) => editor_changed(markdown, runtime),
        UserEvent::ShowAbout => render_about(&runtime.webview),
        UserEvent::OpenDocumentation => open_documentation(runtime),
        UserEvent::Exit => exit_application(runtime, control_flow),
    }
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
            let message =
                text("status.failed_rebuild_menu").replace("{error}", &error.to_string());
            render_status(&runtime.webview, &message, "error");
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
    }
    let ready = text("status.ready");
    runtime.native_footer.sync(&runtime.workspace, ready);
    render_status(&runtime.webview, ready, "info");
}

#[cfg(target_os = "macos")]
fn sync_native_menu_state_after_language_change(runtime: &AppRuntime) {
    super::workspace_view::sync_native_menu_state(&runtime.workspace);
    sync_native_theme_state(runtime.selected_theme);
    sync_native_language_state(runtime.language);
}

fn handle_open_file(ctx: super::ActionContext, runtime: &mut AppRuntime) {
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
                let before_active = runtime.workspace.active_document_id();
                open_document(
                    &runtime.window,
                    &runtime.webview,
                    &runtime.native_footer,
                    &mut runtime.workspace,
                    &path,
                    Some(ctx.event_id),
                    &runtime.asset_access,
                );
                let after_active = runtime.workspace.active_document_id();
                if after_active == before_active && runtime.workspace.find_by_path(&path).is_none()
                {
                    failures += 1;
                }
            }
            if failures > 0 {
                render_status(&runtime.webview, text("status.some_open_failed"), "error");
            }
        }
        None => render_status(&runtime.webview, text("status.open_cancelled"), "info"),
    }
}

fn handle_open_path(request: OpenPathRequest, runtime: &mut AppRuntime) {
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
    open_document(
        &runtime.window,
        &runtime.webview,
        &runtime.native_footer,
        &mut runtime.workspace,
        &path,
        Some(ctx.event_id),
        &runtime.asset_access,
    );
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
        None => render_status(&runtime.webview, text("status.no_document"), "error"),
    }
}

fn select_theme(theme: AppTheme, runtime: &mut AppRuntime) {
    runtime.selected_theme = theme;
    theme_preferences::save_selected_theme(theme);
    runtime.native_footer.set_theme(theme);
    sync_native_theme_state(theme);
    let css = render_assets::load_app_theme_css_for_inline_style(theme.key());
    match serde_json::to_string(&css) {
        Ok(serialized_css) => {
            let script = format!("window.applyAppTheme({serialized_css});");
            if let Err(error) = runtime.webview.evaluate_script(&script) {
                let message =
                    text("status.failed_apply_theme").replace("{error}", &error.to_string());
                render_status(&runtime.webview, &message, "error");
                return;
            }
            let theme_name = match theme {
                AppTheme::Default => text("menu.theme_default"),
                AppTheme::Dark => text("menu.theme_dark"),
                AppTheme::Light => text("menu.theme_light"),
            };
            let message = text("status.theme_switched").replace("{theme}", theme_name);
            render_status(&runtime.webview, &message, "info");
        }
        Err(error) => {
            let message =
                text("status.failed_serialize_theme").replace("{error}", &error.to_string());
            render_status(&runtime.webview, &message, "error");
        }
    }
}

fn update_document_size(size: crate::app::DocumentSize, action: &str, runtime: &mut AppRuntime) {
    runtime.document_size = size;
    theme_preferences::save_document_size(size);
    let script = format!("window.applyDocumentSize({});", size.percent());
    if let Err(error) = runtime.webview.evaluate_script(&script) {
        let message = text("status.failed_apply_size").replace("{error}", &error.to_string());
        render_status(&runtime.webview, &message, "error");
        return;
    }
    let status = text("status.size_changed")
        .replace("{action}", action)
        .replace("{percent}", &size.percent().to_string());
    runtime.native_footer.set_status(&status);
    render_status(&runtime.webview, &status, "info");
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
            runtime.native_footer.set_status(&status);
            render_status(&runtime.webview, &status, "error");
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
    let message = if runtime.window.fullscreen().is_some() {
        text("status.entered_fullscreen")
    } else {
        text("status.exited_fullscreen")
    };
    render_status(&runtime.webview, message, "info");
}

fn editor_changed(markdown: String, runtime: &mut AppRuntime) {
    if let Some(document) = runtime.workspace.active_document_mut() {
        document.update_markdown(markdown);
        sync_workspace_state(
            &runtime.window,
            &runtime.webview,
            &runtime.native_footer,
            &runtime.workspace,
            text("status.unsaved"),
        );
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
        render_status(&runtime.webview, &message, "error");
    }
}
