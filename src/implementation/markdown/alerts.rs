use pulldown_cmark::BlockQuoteKind;

use super::escape_html;

/// A GitHub-compatible Markdown Alert type.
///
/// The parser owns marker recognition through `Options::ENABLE_GFM`. This module owns the
/// rendered structure, which stays identical across the app, PNG, PDF, standalone HTML, and
/// native Print so a single DOM contract covers every output path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AlertKind {
    pub(super) const fn from_block_quote_kind(kind: BlockQuoteKind) -> Self {
        match kind {
            BlockQuoteKind::Note => Self::Note,
            BlockQuoteKind::Tip => Self::Tip,
            BlockQuoteKind::Important => Self::Important,
            BlockQuoteKind::Warning => Self::Warning,
            BlockQuoteKind::Caution => Self::Caution,
        }
    }

    /// Stable type token. This is the single styling and localization switch; CSS selects on it
    /// and the shared localization payload keys off it.
    const fn token(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }

    /// Decorative icon geometry. Inlined per Alert so every static export stays self-contained
    /// without a shared sprite definition.
    const fn icon_body(self) -> &'static str {
        match self {
            Self::Note => {
                "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 16v-5\"/><path d=\"M12 8h.01\"/>"
            }
            Self::Tip => {
                "<path d=\"M9 18h6\"/><path d=\"M10 21h4\"/><path d=\"M12 3a6 6 0 0 0-3.6 10.8c.5.4.8 1 .9 1.6l.1.6h5.2l.1-.6c.1-.6.4-1.2.9-1.6A6 6 0 0 0 12 3z\"/>"
            }
            Self::Important => {
                "<path d=\"M21 11.5a8.4 8.4 0 0 1-9 8.4L4 21l1.1-3.6A8.4 8.4 0 1 1 21 11.5z\"/><path d=\"M12 8v4\"/><path d=\"M12 15h.01\"/>"
            }
            Self::Warning => {
                "<path d=\"M10.3 3.9 1.9 18a2 2 0 0 0 1.7 3h16.8a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z\"/><path d=\"M12 9v4\"/><path d=\"M12 17h.01\"/>"
            }
            Self::Caution => {
                "<path d=\"M7.9 2h8.2L22 7.9v8.2L16.1 22H7.9L2 16.1V7.9L7.9 2z\"/><path d=\"M12 8v4.5\"/><path d=\"M12 16h.01\"/>"
            }
        }
    }
}

/// Visible type labels for the five Alert types.
///
/// Markdown rendering owns the structure, not the interface language, so callers supply the text.
/// The app renders with the English defaults and localizes in the WebView, which keeps titles
/// correct when the language changes while a document is open. Export paths inject the current
/// language up front so a static PNG, PDF, or standalone HTML file is already correct without
/// running any script.
#[derive(Clone, Copy, Debug)]
pub struct AlertLabels {
    pub note: &'static str,
    pub tip: &'static str,
    pub important: &'static str,
    pub warning: &'static str,
    pub caution: &'static str,
}

impl AlertLabels {
    /// English defaults, also used by offline CLI and socket output so those paths stay
    /// deterministic instead of reading interface preferences.
    pub const fn english() -> Self {
        Self {
            note: "Note",
            tip: "Tip",
            important: "Important",
            warning: "Warning",
            caution: "Caution",
        }
    }

    fn label_for(self, kind: AlertKind) -> &'static str {
        match kind {
            AlertKind::Note => self.note,
            AlertKind::Tip => self.tip,
            AlertKind::Important => self.important,
            AlertKind::Warning => self.warning,
            AlertKind::Caution => self.caution,
        }
    }
}

/// Opening markup for one Alert.
///
/// `id_number` must be unique within the document. Alert identifiers use their own
/// `markhola-alert-` namespace so they cannot collide with heading slugs or footnote anchors.
pub(super) fn open_html(kind: AlertKind, id_number: usize, labels: AlertLabels) -> String {
    let token = kind.token();
    let title_id = format!("markhola-alert-{id_number}");
    format!(
        "<div class=\"mh-alert\" data-alert=\"{token}\" role=\"note\" aria-labelledby=\"{title_id}\">\
<div class=\"mh-alert-title\" id=\"{title_id}\">\
<svg class=\"mh-alert-icon\" aria-hidden=\"true\" focusable=\"false\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>\
<span class=\"mh-alert-label\">{}</span>\
</div>\
<div class=\"mh-alert-body\">",
        kind.icon_body(),
        escape_html(labels.label_for(kind))
    )
}

/// Closing markup for one Alert. Must pair with every `open_html` call.
pub(super) const fn close_html() -> &'static str {
    "</div></div>"
}
