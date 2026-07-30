mod asset_access;
mod bootstrap;
mod close_actions;
mod document_actions;
mod document_surface;
mod documentation;
mod event_loop;
mod export_actions;
mod ipc;
mod native_footer;
mod native_tabs;
mod navigation_actions;
mod protocol_commands;
mod protocol_transport;
mod runtime;
mod save_actions;
mod shell_events;
mod shortcuts;
mod surface_actions;
mod theme_actions;
mod theme_preferences;
mod user_events;
mod window_events;
mod workspace_view;

#[allow(unused_imports)]
pub(crate) use self::document_actions::{load_document, reload_workspace_documents_from_disk};
#[allow(unused_imports)]
pub(crate) use self::event_loop::file_paths_from_urls;
#[allow(unused_imports)]
pub(crate) use self::ipc::markdown_path_from_href;

use super::*;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    install_panic_hook();
    log_event(
        "app.start",
        None,
        "app run started",
        format!("version={APP_VERSION} platform={APP_BUILD_PLATFORM}/{APP_BUILD_TARGET}"),
    );

    let (event_loop, mut runtime) = bootstrap::build_runtime()?;
    workspace_view::sync_native_menu_state(&runtime.workspace);
    workspace_view::sync_native_theme_state(runtime.theme_preference);
    workspace_view::sync_native_language_state(runtime.language);

    event_loop.run(move |event, target, control_flow| {
        event_loop::handle_event(event, target, &mut runtime, control_flow);
    });

    #[allow(unreachable_code)]
    Ok(())
}
