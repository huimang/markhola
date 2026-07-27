use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tao::keyboard::ModifiersState;
use tao::window::{Window, WindowId};
use wry::WebView;

use super::native_footer::NativeFooter;
use super::runtime::ShellRuntime;

pub(super) struct DocumentSurface {
    pub(super) window: Window,
    pub(super) webview: WebView,
    pub(super) modifiers: ModifiersState,
    pub(super) shell: ShellRuntime,
    pub(super) native_footer: NativeFooter,
    pub(super) document_id: Option<u64>,
}

impl DocumentSurface {
    pub(super) fn new(
        window: Window,
        webview: WebView,
        suppress_blank_recovery: Arc<AtomicBool>,
        native_footer: NativeFooter,
        document_id: Option<u64>,
    ) -> Self {
        Self {
            window,
            webview,
            modifiers: ModifiersState::default(),
            shell: ShellRuntime::new(suppress_blank_recovery),
            native_footer,
            document_id,
        }
    }

    pub(super) fn window_id(&self) -> WindowId {
        self.window.id()
    }
}
