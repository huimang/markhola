use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tao::event_loop::EventLoopProxy;
use tao::keyboard::ModifiersState;
use tao::window::{Window, WindowId};
use wry::WebView;

use crate::app::{AppLanguage, AppTheme, DocumentSize, ThemePreference};
use crate::workspace::DocumentWorkspace;

use super::asset_access::AssetAccessRegistry;
use super::document_surface::DocumentSurface;
use super::native_footer::NativeFooter;
use super::protocol_commands::ProtocolCommandRuntime;
use super::protocol_transport::ProtocolTransport;
use super::{OpenPathRequest, UserEvent};

pub(super) struct ShellRuntime {
    pub(super) ready: bool,
    pub(super) recovery_pending: bool,
    pub(super) pending_open_requests: Vec<OpenPathRequest>,
    pub(super) suppress_blank_recovery: Arc<AtomicBool>,
}

impl ShellRuntime {
    pub(super) fn new(suppress_blank_recovery: Arc<AtomicBool>) -> Self {
        Self {
            ready: false,
            recovery_pending: false,
            pending_open_requests: Vec::new(),
            suppress_blank_recovery,
        }
    }
}

pub(super) struct AppRuntime {
    pub(super) proxy: EventLoopProxy<UserEvent>,
    pub(super) window: Window,
    pub(super) webview: WebView,
    pub(super) workspace: DocumentWorkspace,
    pub(super) modifiers: ModifiersState,
    pub(super) shell: ShellRuntime,
    pub(super) asset_access: AssetAccessRegistry,
    pub(super) selected_theme: AppTheme,
    pub(super) theme_preference: ThemePreference,
    pub(super) language: AppLanguage,
    pub(super) document_size: DocumentSize,
    pub(super) native_footer: NativeFooter,
    pub(super) active_document_id: Option<u64>,
    pub(super) inactive_surfaces: HashMap<WindowId, DocumentSurface>,
    pub(super) _protocol_transport: ProtocolTransport,
    pub(super) protocol_commands: ProtocolCommandRuntime,
}

impl AppRuntime {
    pub(super) fn new(
        proxy: EventLoopProxy<UserEvent>,
        window: Window,
        webview: WebView,
        suppress_blank_recovery: Arc<AtomicBool>,
        asset_access: AssetAccessRegistry,
        native_footer: NativeFooter,
        selected_theme: AppTheme,
        theme_preference: ThemePreference,
        language: AppLanguage,
        document_size: DocumentSize,
        protocol_transport: ProtocolTransport,
        protocol_commands: ProtocolCommandRuntime,
    ) -> Self {
        Self {
            proxy,
            window,
            webview,
            workspace: DocumentWorkspace::new(),
            modifiers: ModifiersState::default(),
            shell: ShellRuntime::new(suppress_blank_recovery),
            asset_access,
            selected_theme,
            theme_preference,
            language,
            document_size,
            native_footer,
            active_document_id: None,
            inactive_surfaces: HashMap::new(),
            _protocol_transport: protocol_transport,
            protocol_commands,
        }
    }

    pub(super) fn active_window_id(&self) -> WindowId {
        self.window.id()
    }

    pub(super) fn document_id_for_window(&self, window_id: WindowId) -> Option<u64> {
        if self.window.id() == window_id {
            self.active_document_id
        } else {
            self.inactive_surfaces
                .get(&window_id)
                .and_then(|surface| surface.document_id)
        }
    }

    pub(super) fn window_id_for_document(&self, document_id: u64) -> Option<WindowId> {
        if self.active_document_id == Some(document_id) {
            return Some(self.window.id());
        }
        self.inactive_surfaces
            .values()
            .find(|surface| surface.document_id == Some(document_id))
            .map(DocumentSurface::window_id)
    }

    pub(super) fn insert_surface(&mut self, surface: DocumentSurface) {
        self.inactive_surfaces.insert(surface.window_id(), surface);
    }

    pub(super) fn activate_surface(&mut self, window_id: WindowId) -> bool {
        if self.window.id() == window_id {
            return true;
        }
        let Some(surface) = self.inactive_surfaces.remove(&window_id) else {
            return false;
        };

        super::native_tabs::sync_zoom_state(&self.window, &surface.window);
        let DocumentSurface {
            window,
            webview,
            modifiers,
            shell,
            native_footer,
            document_id,
        } = surface;
        let previous = DocumentSurface {
            window: std::mem::replace(&mut self.window, window),
            webview: std::mem::replace(&mut self.webview, webview),
            modifiers: std::mem::replace(&mut self.modifiers, modifiers),
            shell: std::mem::replace(&mut self.shell, shell),
            native_footer: std::mem::replace(&mut self.native_footer, native_footer),
            document_id: std::mem::replace(&mut self.active_document_id, document_id),
        };
        self.inactive_surfaces
            .insert(previous.window_id(), previous);
        true
    }

    pub(super) fn remove_inactive_surface_for_document(
        &mut self,
        document_id: u64,
    ) -> Option<DocumentSurface> {
        let window_id = self
            .inactive_surfaces
            .iter()
            .find_map(|(window_id, surface)| {
                (surface.document_id == Some(document_id)).then_some(*window_id)
            })?;
        self.inactive_surfaces.remove(&window_id)
    }

    pub(super) fn reset_to_empty_surface(&mut self) {
        self.active_document_id = None;
        self.inactive_surfaces.clear();
    }
}
