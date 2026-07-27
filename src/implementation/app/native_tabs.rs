#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSColor, NSWindow, NSWindowOrderingMode, NSWindowTabbingMode,
};
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowExtMacOS;
use tao::window::Window;
use tao::window::WindowId;

use super::runtime::AppRuntime;
use crate::app::AppTheme;

const TABBING_IDENTIFIER: &str = "com.markhola.document-tabs";

pub(super) fn configure(window: &Window, theme: AppTheme) {
    #[cfg(target_os = "macos")]
    {
        window.set_allows_automatic_window_tabbing(false);
        window.set_tabbing_identifier(TABBING_IDENTIFIER);
        window.set_titlebar_transparent(true);
        unsafe {
            let ns_window = &*(window.ns_window() as *mut NSWindow);
            ns_window.setTabbingMode(NSWindowTabbingMode::Preferred);
        }
        apply_theme(window, theme);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (window, theme);
}

pub(super) fn add_to_group(anchor: &Window, window: &Window) {
    #[cfg(target_os = "macos")]
    unsafe {
        let anchor = &*(anchor.ns_window() as *mut NSWindow);
        let added = &*(window.ns_window() as *mut NSWindow);
        anchor.addTabbedWindow_ordered(added, NSWindowOrderingMode::Above);
    }
    window.set_visible(true);
    window.set_focus();
}

pub(super) fn select(window: &Window) {
    window.set_visible(true);
    window.set_focus();
}

pub(super) fn select_window_id(runtime: &AppRuntime, window_id: WindowId) {
    if runtime.window.id() == window_id {
        select(&runtime.window);
        return;
    }
    if let Some(surface) = runtime.inactive_surfaces.get(&window_id) {
        select(&surface.window);
    }
}

pub(super) fn visible_document_ids(runtime: &AppRuntime) -> Vec<u64> {
    #[cfg(target_os = "macos")]
    unsafe {
        let active_window = &*(runtime.window.ns_window() as *mut NSWindow);
        if let Some(tabbed_windows) = active_window.tabbedWindows() {
            return tabbed_windows
                .iter()
                .filter_map(|window| {
                    document_id_for_native_window(runtime, Retained::as_ptr(&window))
                })
                .collect();
        }
    }

    runtime
        .workspace
        .tab_snapshots()
        .into_iter()
        .map(|tab| tab.document_id)
        .collect()
}

#[cfg(target_os = "macos")]
fn document_id_for_native_window(
    runtime: &AppRuntime,
    native_window: *const NSWindow,
) -> Option<u64> {
    if std::ptr::eq(runtime.window.ns_window() as *const NSWindow, native_window) {
        return runtime.active_document_id;
    }
    runtime
        .inactive_surfaces
        .values()
        .find(|surface| std::ptr::eq(surface.window.ns_window() as *const NSWindow, native_window))
        .and_then(|surface| surface.document_id)
}

pub(super) fn set_document_edited(window: &Window, edited: bool) {
    #[cfg(target_os = "macos")]
    window.set_is_document_edited(edited);
    #[cfg(not(target_os = "macos"))]
    let _ = (window, edited);
}

pub(super) fn apply_theme(window: &Window, theme: AppTheme) {
    #[cfg(target_os = "macos")]
    unsafe {
        let ns_window = &*(window.ns_window() as *mut NSWindow);
        let appearance_name = if theme == AppTheme::Dark {
            NSAppearanceNameDarkAqua
        } else {
            NSAppearanceNameAqua
        };
        if let Some(appearance) = NSAppearance::appearanceNamed(appearance_name) {
            ns_window.setAppearance(Some(&appearance));
        }
        let (red, green, blue) = match theme {
            AppTheme::Default => (234, 249, 245),
            AppTheme::Light => (234, 249, 245),
            AppTheme::Dark => (36, 33, 58),
        };
        let color = NSColor::colorWithSRGBRed_green_blue_alpha(
            red as f64 / 255.0,
            green as f64 / 255.0,
            blue as f64 / 255.0,
            1.0,
        );
        ns_window.setBackgroundColor(Some(&color));
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (window, theme);
}
