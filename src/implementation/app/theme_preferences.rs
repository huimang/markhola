use crate::app::{AppTheme, DocumentSize};

const SELECTED_THEME_KEY: &str = "selectedTheme";
const DOCUMENT_SIZE_KEY: &str = "documentSizePercent";

#[cfg(target_os = "macos")]
pub(super) fn load_selected_theme() -> AppTheme {
    use objc2_foundation::{NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    defaults
        .stringForKey(ns_string!(SELECTED_THEME_KEY))
        .and_then(|value| AppTheme::from_key(value.to_string().as_str()))
        .unwrap_or(AppTheme::Default)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn load_selected_theme() -> AppTheme {
    AppTheme::Default
}

#[cfg(target_os = "macos")]
pub(super) fn save_selected_theme(theme: AppTheme) {
    use objc2_foundation::{NSString, NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    let value = NSString::from_str(theme.key());
    unsafe {
        defaults.setObject_forKey(Some(&*value), ns_string!(SELECTED_THEME_KEY));
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn save_selected_theme(_theme: AppTheme) {}

#[cfg(target_os = "macos")]
pub(super) fn load_document_size() -> DocumentSize {
    use objc2_foundation::{NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    DocumentSize::from_stored(defaults.integerForKey(ns_string!(DOCUMENT_SIZE_KEY)) as i64)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn load_document_size() -> DocumentSize {
    DocumentSize::default()
}

#[cfg(target_os = "macos")]
pub(super) fn save_document_size(size: DocumentSize) {
    use objc2_foundation::{NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    defaults.setInteger_forKey(size.percent() as isize, ns_string!(DOCUMENT_SIZE_KEY));
}

#[cfg(not(target_os = "macos"))]
pub(super) fn save_document_size(_size: DocumentSize) {}
