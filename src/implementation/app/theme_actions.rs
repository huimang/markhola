use tao::window::Theme;

use crate::app::{AppTheme, ThemePreference, text};
use crate::render_assets;

use super::native_tabs;
use super::runtime::AppRuntime;
use super::theme_preferences;
use super::workspace_view::{render_status, sync_native_theme_state};

pub(super) fn select_theme(preference: ThemePreference, runtime: &mut AppRuntime) {
    runtime.theme_preference = preference;
    theme_preferences::save_theme_preference(preference);
    sync_native_theme_state(preference);
    let theme = preference.resolve(runtime.window.theme());
    apply_effective_theme(theme, runtime, true);
}

pub(super) fn system_theme_changed(system_theme: Theme, runtime: &mut AppRuntime) {
    if runtime.theme_preference != ThemePreference::System {
        return;
    }
    let theme = runtime.theme_preference.resolve(system_theme);
    if theme != runtime.selected_theme {
        apply_effective_theme(theme, runtime, false);
    }
}

fn apply_effective_theme(theme: AppTheme, runtime: &mut AppRuntime, show_status: bool) {
    runtime.selected_theme = theme;
    runtime.native_footer.set_theme(theme);
    native_tabs::apply_theme(&runtime.window, theme);
    for surface in runtime.inactive_surfaces.values() {
        surface.native_footer.set_theme(theme);
        native_tabs::apply_theme(&surface.window, theme);
    }

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
            for surface in runtime.inactive_surfaces.values() {
                let _ = surface.webview.evaluate_script(&script);
            }
            if show_status {
                let theme_name = match runtime.theme_preference {
                    ThemePreference::System => text("menu.theme_system"),
                    ThemePreference::Default => text("menu.theme_default"),
                    ThemePreference::Dark => text("menu.theme_dark"),
                };
                let message = text("status.theme_switched").replace("{theme}", theme_name);
                render_status(&runtime.webview, &message, "info");
            }
        }
        Err(error) => {
            let message =
                text("status.failed_serialize_theme").replace("{error}", &error.to_string());
            render_status(&runtime.webview, &message, "error");
        }
    }
}
