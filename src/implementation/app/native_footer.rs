use tao::window::Window;
use wry::WebView;

use crate::app::{AppTheme, text};
use crate::document::DocumentSnapshot;
use crate::workspace::DocumentWorkspace;

#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2::MainThreadOnly;
#[cfg(target_os = "macos")]
use objc2::msg_send;
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSColor, NSFont, NSTextAlignment, NSTextField, NSView, NSWindow,
};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowExtMacOS;
#[cfg(target_os = "macos")]
use wry::WebViewExtMacOS;

const FOOTER_HEIGHT: f64 = 34.0;
const FOOTER_PADDING_X: f64 = 16.0;
const FOOTER_LABEL_Y: f64 = 7.0;
const FOOTER_LABEL_HEIGHT: f64 = 18.0;
const FOOTER_GAP: f64 = 10.0;
const FOOTER_LINES_WIDTH: f64 = 78.0;
const FOOTER_WORDS_WIDTH: f64 = 82.0;

pub(super) struct NativeFooter {
    #[cfg(target_os = "macos")]
    handle: Option<NativeFooterHandle>,
}

#[cfg(target_os = "macos")]
struct NativeFooterHandle {
    footer_view: Retained<NSView>,
    path_field: Retained<NSTextField>,
    words_field: Retained<NSTextField>,
    lines_field: Retained<NSTextField>,
}

impl NativeFooter {
    pub(super) fn install(window: &Window, webview: &WebView, theme: AppTheme) -> Self {
        #[cfg(target_os = "macos")]
        unsafe {
            let Some(mtm) = MainThreadMarker::new() else {
                return Self { handle: None };
            };
            let ns_window = &*(window.ns_window() as *mut NSWindow);
            let Some(content_view) = ns_window.contentView() else {
                return Self { handle: None };
            };

            let footer_view = NSView::initWithFrame(
                NSView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(100.0, FOOTER_HEIGHT)),
            );
            footer_view.setWantsLayer(true);
            footer_view.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewMaxYMargin,
            );

            let path_field = footer_label(mtm, "");
            let words_field = footer_label(mtm, "");
            let lines_field = footer_label(mtm, "");
            words_field.setAlignment(NSTextAlignment::Right);
            lines_field.setAlignment(NSTextAlignment::Right);

            apply_footer_fonts(&[&path_field, &words_field, &lines_field]);

            footer_view.addSubview(&path_field);
            footer_view.addSubview(&words_field);
            footer_view.addSubview(&lines_field);
            content_view.addSubview(&footer_view);

            let handle = NativeFooterHandle {
                footer_view,
                path_field,
                words_field,
                lines_field,
            };

            let footer = Self {
                handle: Some(handle),
            };
            footer.set_theme(theme);
            footer.relayout(window, webview);
            footer
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (window, webview, theme);
            Self {}
        }
    }

    pub(super) fn set_theme(&self, theme: AppTheme) {
        #[cfg(target_os = "macos")]
        {
            let Some(handle) = &self.handle else {
                return;
            };
            let (background, foreground) = footer_theme_colors(theme);
            if let Some(layer) = handle.footer_view.layer() {
                layer.setBackgroundColor(Some(&background.CGColor()));
            }
            handle.path_field.setTextColor(Some(&foreground));
            handle.words_field.setTextColor(Some(&foreground));
            handle.lines_field.setTextColor(Some(&foreground));
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = theme;
        }
    }

    pub(super) fn relayout(&self, window: &Window, webview: &WebView) {
        #[cfg(target_os = "macos")]
        unsafe {
            let Some(handle) = &self.handle else {
                return;
            };
            let ns_window = &*(window.ns_window() as *mut NSWindow);
            let Some(content_view) = ns_window.contentView() else {
                return;
            };
            let content_frame = content_view.frame();
            let width = content_frame.size.width;
            let height = content_frame.size.height;
            let footer_height = FOOTER_HEIGHT.min(height.max(0.0));

            handle.footer_view.setFrame(NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(width, footer_height),
            ));

            let lines_x = width - FOOTER_PADDING_X - FOOTER_LINES_WIDTH;
            let words_x = lines_x - FOOTER_GAP - FOOTER_WORDS_WIDTH;
            let path_width = (words_x - FOOTER_GAP - FOOTER_PADDING_X).max(0.0);
            handle.path_field.setFrame(NSRect::new(
                NSPoint::new(FOOTER_PADDING_X, FOOTER_LABEL_Y),
                NSSize::new(path_width, FOOTER_LABEL_HEIGHT),
            ));
            handle.words_field.setFrame(NSRect::new(
                NSPoint::new(words_x, FOOTER_LABEL_Y),
                NSSize::new(FOOTER_WORDS_WIDTH, FOOTER_LABEL_HEIGHT),
            ));
            handle.lines_field.setFrame(NSRect::new(
                NSPoint::new(lines_x, FOOTER_LABEL_Y),
                NSSize::new(FOOTER_LINES_WIDTH, FOOTER_LABEL_HEIGHT),
            ));
            let webview_handle = webview.webview();
            webview_handle.setFrame(NSRect::new(
                NSPoint::new(0.0, footer_height),
                NSSize::new(width, (height - footer_height).max(0.0)),
            ));
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (window, webview);
        }
    }

    pub(super) fn sync(&self, workspace: &DocumentWorkspace, _status: &str) {
        self.sync_document(workspace.active_document_snapshot(), _status);
    }

    pub(super) fn sync_document(&self, document: Option<DocumentSnapshot>, _status: &str) {
        #[cfg(target_os = "macos")]
        unsafe {
            let Some(handle) = &self.handle else {
                return;
            };

            if let Some(active) = document {
                set_label_text(&handle.path_field, &active.file_path);
                set_label_text(
                    &handle.words_field,
                    &format!("{} {}", text("footer.words"), active.word_count),
                );
                set_label_text(
                    &handle.lines_field,
                    &format!("{} {}", text("footer.lines"), active.line_count),
                );
                set_hidden(&handle.path_field, false);
                set_hidden(&handle.words_field, false);
                set_hidden(&handle.lines_field, false);
            } else {
                set_label_text(&handle.path_field, "");
                set_label_text(&handle.words_field, "");
                set_label_text(&handle.lines_field, "");
                set_hidden(&handle.path_field, true);
                set_hidden(&handle.words_field, true);
                set_hidden(&handle.lines_field, true);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (document, _status);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn footer_label(mtm: MainThreadMarker, value: &str) -> Retained<NSTextField> {
    let string = NSString::from_str(value);
    let label = NSTextField::labelWithString(&string, mtm);
    label.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewMaxXMargin | NSAutoresizingMaskOptions::ViewMinYMargin,
    );
    label
}

#[cfg(target_os = "macos")]
fn footer_theme_colors(theme: AppTheme) -> (Retained<NSColor>, Retained<NSColor>) {
    match theme {
        AppTheme::Default => (rgb_color(248, 250, 252), rgb_color(71, 85, 105)),
        AppTheme::Dark => (rgb_color(30, 41, 59), rgb_color(148, 163, 184)),
    }
}

#[cfg(target_os = "macos")]
fn rgb_color(red: u8, green: u8, blue: u8) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        f64::from(red) / 255.0,
        f64::from(green) / 255.0,
        f64::from(blue) / 255.0,
        1.0,
    )
}

#[cfg(target_os = "macos")]
unsafe fn apply_footer_fonts(fields: &[&NSTextField]) {
    let font = NSFont::systemFontOfSize(12.0);
    for field in fields {
        let _: () = msg_send![*field, setFont: Some(&*font)];
    }
}

#[cfg(target_os = "macos")]
unsafe fn set_label_text(field: &NSTextField, value: &str) {
    let string = NSString::from_str(value);
    let _: () = msg_send![field, setStringValue: &*string];
}

#[cfg(target_os = "macos")]
unsafe fn set_hidden(view: &objc2::runtime::AnyObject, hidden: bool) {
    let _: () = msg_send![view, setHidden: hidden];
}
