use std::str::FromStr;

use syntect::highlighting::{
    Color, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings,
};

const TEXT: Color = color(0x01, 0x01, 0x01);
const COMMENT: Color = color(0x02, 0x02, 0x02);
const STRING: Color = color(0x03, 0x03, 0x03);
const CONSTANT: Color = color(0x04, 0x04, 0x04);
const KEYWORD: Color = color(0x05, 0x05, 0x05);
const ENTITY: Color = color(0x06, 0x06, 0x06);

pub(super) fn markhola_theme() -> Theme {
    Theme {
        name: Some("MarkHola".to_string()),
        author: Some("MarkHola".to_string()),
        settings: ThemeSettings {
            foreground: Some(TEXT),
            ..ThemeSettings::default()
        },
        scopes: vec![
            theme_item("comment", COMMENT, Some(FontStyle::ITALIC)),
            theme_item("string, string.regexp", STRING, None),
            theme_item(
                "constant.numeric, constant.language, constant.character, constant.other",
                CONSTANT,
                None,
            ),
            theme_item(
                "keyword, keyword.control, storage, storage.type, storage.modifier",
                KEYWORD,
                None,
            ),
            theme_item(
                "entity.name.function, entity.name.type, entity.other.attribute-name, \
                 support.function, support.type, variable.function, variable.other.member",
                ENTITY,
                None,
            ),
        ],
    }
}

pub(super) fn semantic_class(color: Color) -> &'static str {
    match color {
        COMMENT => "comment",
        STRING => "string",
        CONSTANT => "constant",
        KEYWORD => "keyword",
        ENTITY => "entity",
        _ => "text",
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
