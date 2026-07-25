use std::str::FromStr;

use syntect::highlighting::{
    Color, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings,
};

const VIOLET_MID: Color = color(0xAC, 0xA4, 0xF4);
const VIOLET_TINT: Color = color(0xF2, 0xF0, 0xFF);
const GREEN_MID: Color = color(0x81, 0xD9, 0xC3);
const GREEN_TINT: Color = color(0xEA, 0xF9, 0xF5);
const CODE_SURFACE: Color = color(0x24, 0x21, 0x3A);

pub(super) fn markhola_theme() -> Theme {
    Theme {
        name: Some("MarkHola".to_string()),
        author: Some("MarkHola".to_string()),
        settings: ThemeSettings {
            foreground: Some(VIOLET_TINT),
            background: Some(CODE_SURFACE),
            accent: Some(GREEN_MID),
            ..ThemeSettings::default()
        },
        scopes: vec![
            theme_item("comment", GREEN_MID, Some(FontStyle::ITALIC)),
            theme_item("string, string.regexp", GREEN_MID, None),
            theme_item(
                "constant.numeric, constant.language, constant.character, constant.other",
                GREEN_TINT,
                None,
            ),
            theme_item(
                "keyword, keyword.control, storage, storage.type, storage.modifier",
                VIOLET_MID,
                Some(FontStyle::BOLD),
            ),
            theme_item(
                "entity.name.function, entity.name.type, entity.other.attribute-name, \
                 support.function, support.type, variable.function, variable.other.member",
                VIOLET_TINT,
                None,
            ),
        ],
    }
}

const fn color(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 0xFF }
}

fn theme_item(scopes: &str, foreground: Color, font_style: Option<FontStyle>) -> ThemeItem {
    ThemeItem {
        scope: ScopeSelectors::from_str(scopes).expect("MarkHola syntax scopes should be valid"),
        style: StyleModifier {
            foreground: Some(foreground),
            background: None,
            font_style,
        },
    }
}
