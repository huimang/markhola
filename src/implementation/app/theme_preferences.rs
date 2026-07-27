use crate::app::{AppLanguage, DocumentSize, ThemePreference};

const SELECTED_THEME_KEY: &str = "selectedTheme";
const DOCUMENT_SIZE_KEY: &str = "documentSizePercent";
const APP_LANGUAGE_KEY: &str = "appLanguage";

#[cfg(target_os = "macos")]
pub(super) fn load_app_language() -> AppLanguage {
    use objc2_foundation::{NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    defaults
        .stringForKey(ns_string!(APP_LANGUAGE_KEY))
        .and_then(|value| AppLanguage::from_key(value.to_string().as_str()))
        .unwrap_or(AppLanguage::English)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn load_app_language() -> AppLanguage {
    AppLanguage::English
}

#[cfg(target_os = "macos")]
pub(super) fn save_app_language(language: AppLanguage) {
    use objc2_foundation::{NSString, NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    let value = NSString::from_str(language.key());
    unsafe {
        defaults.setObject_forKey(Some(&*value), ns_string!(APP_LANGUAGE_KEY));
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn save_app_language(_language: AppLanguage) {}

#[cfg(target_os = "macos")]
pub(super) fn load_theme_preference() -> ThemePreference {
    use objc2_foundation::{NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    defaults
        .stringForKey(ns_string!(SELECTED_THEME_KEY))
        .and_then(|value| ThemePreference::from_stored_key(value.to_string().as_str()))
        .unwrap_or(ThemePreference::System)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn load_theme_preference() -> ThemePreference {
    ThemePreference::System
}

#[cfg(target_os = "macos")]
pub(super) fn save_theme_preference(preference: ThemePreference) {
    use objc2_foundation::{NSString, NSUserDefaults, ns_string};

    let defaults = NSUserDefaults::standardUserDefaults();
    let value = NSString::from_str(preference.key());
    unsafe {
        defaults.setObject_forKey(Some(&*value), ns_string!(SELECTED_THEME_KEY));
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn save_theme_preference(_preference: ThemePreference) {}

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
