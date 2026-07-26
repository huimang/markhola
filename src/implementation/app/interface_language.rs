use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use yaml_rust::{Yaml, YamlLoader};

const ENGLISH_YAML: &str = include_str!("../../../i18n/en.yaml");
const SIMPLIFIED_CHINESE_YAML: &str = include_str!("../../../i18n/zh-CN.yaml");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppLanguage {
    English,
    SimplifiedChinese,
}

impl AppLanguage {
    pub(crate) const ALL: [Self; 2] = [Self::English, Self::SimplifiedChinese];

    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::SimplifiedChinese => "zh-CN",
        }
    }

    pub(crate) fn from_key(value: &str) -> Option<Self> {
        match value {
            "en" => Some(Self::English),
            "zh-CN" => Some(Self::SimplifiedChinese),
            _ => None,
        }
    }
}

static CURRENT_LANGUAGE: AtomicU8 = AtomicU8::new(0);
static ENGLISH_CATALOG: OnceLock<HashMap<String, String>> = OnceLock::new();
static CHINESE_CATALOG: OnceLock<HashMap<String, String>> = OnceLock::new();

pub(crate) fn set_current_language(language: AppLanguage) {
    CURRENT_LANGUAGE.store(language as u8, Ordering::Relaxed);
}

pub(crate) fn current_language() -> AppLanguage {
    match CURRENT_LANGUAGE.load(Ordering::Relaxed) {
        1 => AppLanguage::SimplifiedChinese,
        _ => AppLanguage::English,
    }
}

pub(crate) fn text(key: &'static str) -> &'static str {
    text_for(current_language(), key)
}

pub(crate) fn text_for(language: AppLanguage, key: &'static str) -> &'static str {
    let selected = catalog(language);
    selected
        .get(key)
        .or_else(|| catalog(AppLanguage::English).get(key))
        .map(String::as_str)
        .unwrap_or(key)
}

fn catalog(language: AppLanguage) -> &'static HashMap<String, String> {
    match language {
        AppLanguage::English => ENGLISH_CATALOG.get_or_init(|| parse_catalog(ENGLISH_YAML, "en")),
        AppLanguage::SimplifiedChinese => {
            CHINESE_CATALOG.get_or_init(|| parse_catalog(SIMPLIFIED_CHINESE_YAML, "zh-CN"))
        }
    }
}

fn parse_catalog(source: &str, language: &str) -> HashMap<String, String> {
    let documents = YamlLoader::load_from_str(source)
        .unwrap_or_else(|error| panic!("invalid {language} translation YAML: {error}"));
    let root = documents
        .first()
        .unwrap_or_else(|| panic!("empty {language} translation YAML"));
    let mut values = HashMap::new();
    flatten_yaml(root, "", &mut values);
    values
}

fn flatten_yaml(node: &Yaml, prefix: &str, values: &mut HashMap<String, String>) {
    let Yaml::Hash(entries) = node else {
        return;
    };
    for (raw_key, value) in entries {
        let Some(segment) = raw_key.as_str() else {
            continue;
        };
        let key = if prefix.is_empty() {
            segment.to_string()
        } else {
            format!("{prefix}.{segment}")
        };
        match value {
            Yaml::String(value) => {
                values.insert(key, value.clone());
            }
            Yaml::Hash(_) => flatten_yaml(value, &key, values),
            _ => panic!("translation value for {key} must be a string or mapping"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppLanguage, catalog, text_for};

    #[test]
    fn language_keys_round_trip_and_default_is_explicit() {
        for language in AppLanguage::ALL {
            assert_eq!(AppLanguage::from_key(language.key()), Some(language));
        }
        assert_eq!(AppLanguage::from_key("unknown"), None);
        assert_eq!(AppLanguage::English.key(), "en");
    }

    #[test]
    fn catalogs_have_identical_non_empty_keys() {
        let english = catalog(AppLanguage::English);
        let chinese = catalog(AppLanguage::SimplifiedChinese);
        assert_eq!(english.len(), chinese.len());
        for (key, english_value) in english {
            assert!(!english_value.trim().is_empty(), "{key}");
            assert!(
                chinese
                    .get(key)
                    .is_some_and(|value| !value.trim().is_empty()),
                "{key}"
            );
        }
        assert_eq!(text_for(AppLanguage::English, "menu.file"), "File");
        assert_eq!(
            text_for(AppLanguage::SimplifiedChinese, "menu.file"),
            "文件"
        );
    }
}
