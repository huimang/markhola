use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadOnly, define_class};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use tao::event_loop::EventLoopProxy;

use crate::app::{AppLanguage, ThemePreference};
use crate::app::{UserEvent, dispatch_user_event, log_event, new_action_context};

#[derive(Debug)]
struct ProxyIvars {
    proxy: EventLoopProxy<UserEvent>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = ProxyIvars]
    struct MenuTarget;

unsafe impl NSObjectProtocol for MenuTarget {}

    impl MenuTarget {
        #[unsafe(method(newMenuDocument:))]
        fn new_menu_document(&self, _sender: Option<&AnyObject>) {
            emit(
                &self.ivars().proxy,
                UserEvent::NewDocument,
                "newMenuDocument:",
            );
        }

        #[unsafe(method(openMenuDocument:))]
        fn open_menu_document(&self, _sender: Option<&AnyObject>) {
            let ctx = new_action_context("macos-menu-open");
            log_event("macos.menu.action", Some(ctx.event_id), "macOS menu action openMenuDocument:", "");
            dispatch_user_event(&self.ivars().proxy, "macos-menu", UserEvent::OpenFile(ctx));
        }

        #[unsafe(method(saveMenuDocument:))]
        fn save_menu_document(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::SaveDocument, "saveMenuDocument:"); }
        #[unsafe(method(saveMenuDocumentAs:))]
        fn save_menu_document_as(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::SaveDocumentAs, "saveMenuDocumentAs:"); }
        #[unsafe(method(exportPdfDocument:))]
        fn export_pdf_document(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ExportPdf, "exportPdfDocument:"); }
        #[unsafe(method(exportHtmlDocument:))]
        fn export_html_document(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ExportHtml, "exportHtmlDocument:"); }
        #[unsafe(method(printDocument:))]
        fn print_document(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::PrintDocument, "printDocument:"); }
        #[unsafe(method(openFindPanel:))]
        fn open_find_panel(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::OpenFind, "openFindPanel:"); }
        #[unsafe(method(toggleDocumentMode:))]
        fn toggle_document_mode(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ToggleMode, "toggleDocumentMode:"); }
        #[unsafe(method(selectSystemTheme:))]
        fn select_system_theme(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::SelectTheme(ThemePreference::System), "selectSystemTheme:"); }
        #[unsafe(method(selectDefaultTheme:))]
        fn select_default_theme(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::SelectTheme(ThemePreference::Default), "selectDefaultTheme:"); }
        #[unsafe(method(selectDarkTheme:))]
        fn select_dark_theme(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::SelectTheme(ThemePreference::Dark), "selectDarkTheme:"); }
        #[unsafe(method(selectEnglishLanguage:))]
        fn select_english_language(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::SelectLanguage(AppLanguage::English), "selectEnglishLanguage:"); }
        #[unsafe(method(selectSimplifiedChineseLanguage:))]
        fn select_simplified_chinese_language(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::SelectLanguage(AppLanguage::SimplifiedChinese), "selectSimplifiedChineseLanguage:"); }
        #[unsafe(method(increaseDocumentSize:))]
        fn increase_document_size(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::IncreaseDocumentSize, "increaseDocumentSize:"); }
        #[unsafe(method(decreaseDocumentSize:))]
        fn decrease_document_size(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::DecreaseDocumentSize, "decreaseDocumentSize:"); }
        #[unsafe(method(resetDocumentSize:))]
        fn reset_document_size(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ResetDocumentSize, "resetDocumentSize:"); }
        #[unsafe(method(toggleOutlinePanel:))]
        fn toggle_outline_panel(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ToggleOutline, "toggleOutlinePanel:"); }
        #[unsafe(method(toggleFullscreenWindow:))]
        fn toggle_fullscreen_window(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ToggleFullscreen, "toggleFullscreenWindow:"); }
        #[unsafe(method(closeCurrentDocument:))]
        fn close_current_document(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::CloseCurrentDocument, "closeCurrentDocument:"); }
        #[unsafe(method(activateNextDocument:))]
        fn activate_next_document(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateNextDocument, "activateNextDocument:"); }
        #[unsafe(method(activatePreviousDocument:))]
        fn activate_previous_document(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivatePreviousDocument, "activatePreviousDocument:"); }
        #[unsafe(method(activateDocumentOne:))]
        fn activate_document_one(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(0), "activateDocumentOne:"); }
        #[unsafe(method(activateDocumentTwo:))]
        fn activate_document_two(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(1), "activateDocumentTwo:"); }
        #[unsafe(method(activateDocumentThree:))]
        fn activate_document_three(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(2), "activateDocumentThree:"); }
        #[unsafe(method(activateDocumentFour:))]
        fn activate_document_four(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(3), "activateDocumentFour:"); }
        #[unsafe(method(activateDocumentFive:))]
        fn activate_document_five(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(4), "activateDocumentFive:"); }
        #[unsafe(method(activateDocumentSix:))]
        fn activate_document_six(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(5), "activateDocumentSix:"); }
        #[unsafe(method(activateDocumentSeven:))]
        fn activate_document_seven(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(6), "activateDocumentSeven:"); }
        #[unsafe(method(activateDocumentEight:))]
        fn activate_document_eight(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(7), "activateDocumentEight:"); }
        #[unsafe(method(activateDocumentNine:))]
        fn activate_document_nine(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ActivateDocumentAtIndex(8), "activateDocumentNine:"); }
        #[unsafe(method(closeOtherDocuments:))]
        fn close_other_documents(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::CloseOtherDocuments, "closeOtherDocuments:"); }
        #[unsafe(method(closeAllDocuments:))]
        fn close_all_documents(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::CloseAllDocuments, "closeAllDocuments:"); }
        #[unsafe(method(showAboutPanel:))]
        fn show_about_panel(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::ShowAbout, "showAboutPanel:"); }
        #[unsafe(method(openDocumentation:))]
        fn open_documentation(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::OpenDocumentation, "openDocumentation:"); }
        #[unsafe(method(exitApplication:))]
        fn exit_application(&self, _sender: Option<&AnyObject>) { emit(&self.ivars().proxy, UserEvent::Exit, "exitApplication:"); }
    }
);

impl MenuTarget {
    fn new(mtm: MainThreadMarker, proxy: EventLoopProxy<UserEvent>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ProxyIvars { proxy });
        unsafe { objc2::msg_send![super(this), init] }
    }
}

pub(crate) fn target_ref(
    mtm: MainThreadMarker,
    proxy: EventLoopProxy<UserEvent>,
) -> &'static AnyObject {
    thread_local! {
        static TARGET: Cell<*const AnyObject> = const { Cell::new(std::ptr::null()) };
    }
    TARGET.with(|slot| {
        let existing = slot.get();
        if !existing.is_null() {
            return unsafe { &*existing };
        }
        let target = Box::leak(Box::new(MenuTarget::new(mtm, proxy)));
        let object: &'static AnyObject = (&**target).as_ref();
        slot.set(object as *const AnyObject);
        object
    })
}

fn emit(proxy: &EventLoopProxy<UserEvent>, event: UserEvent, action: &str) {
    log_event(
        "macos.menu.action",
        None,
        &format!("macOS menu action {action}"),
        "",
    );
    dispatch_user_event(proxy, "macos-menu", event);
}
use std::cell::Cell;
