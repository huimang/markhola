use std::fs;
use std::path::PathBuf;

use crate::app::implementation::app_theme::AppTheme;
use crate::app::implementation::interface_document_size::DocumentSize;

const SELECTED_THEME_KEY: &str = "selectedTheme";
const DOCUMENT_SIZE_KEY: &str = "documentSize";

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
    let config_path = config_file_path();
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(theme_key) = config.get(SELECTED_THEME_KEY).and_then(|v| v.as_str()) {
                if let Some(theme) = AppTheme::from_key(theme_key) {
                    return theme;
                }
            }
        }
    }
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
pub(super) fn save_selected_theme(theme: AppTheme) {
    let config_path = config_file_path();
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut config = if let Ok(content) = fs::read_to_string(&config_path) {
        serde_json::from_str::<serde_json::Value>(&content).unwrap_or_default()
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            SELECTED_THEME_KEY.to_string(),
            serde_json::Value::String(theme.key().to_string()),
        );
    }

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = fs::write(&config_path, json);
    }
}

#[cfg(not(target_os = "macos"))]
fn config_file_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| {
        let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
        format!("{}\\AppData\\Roaming", home)
    });
    PathBuf::from(app_data).join("MarkHola").join("config.json")
}

#[cfg(target_os = "macos")]
pub(super) fn save_document_size(_size: DocumentSize) {}

#[cfg(not(target_os = "macos"))]
pub(super) fn save_document_size(size: DocumentSize) {
    let config_path = config_file_path();
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut config = if let Ok(content) = fs::read_to_string(&config_path) {
        serde_json::from_str::<serde_json::Value>(&content).unwrap_or_default()
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            DOCUMENT_SIZE_KEY.to_string(),
            serde_json::Value::Number(serde_json::Number::from(size.percent())),
        );
    }

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = fs::write(&config_path, json);
    }
}

#[cfg(target_os = "macos")]
pub(super) fn load_document_size() -> DocumentSize {
    DocumentSize::default()
}

#[cfg(not(target_os = "macos"))]
pub(super) fn load_document_size() -> DocumentSize {
    let config_path = config_file_path();
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(value) = config.get(DOCUMENT_SIZE_KEY).and_then(|v| v.as_i64()) {
                return DocumentSize::from_stored(value);
            }
        }
    }
    DocumentSize::default()
}
