use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{NSString, ns_string};

use crate::app::{AppLanguage, AppTheme, text};

use super::menu_file_items::action;
use super::menu_state::{
    LanguageMenuSlot, ThemeMenuSlot, remember_language_item, remember_outline_item,
    remember_theme_item,
};

pub(super) fn add_view_menu(mtm: MainThreadMarker, main_menu: &NSMenu, target: &AnyObject) {
    let view_title = NSString::from_str(text("menu.view"));
    let view_menu_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &view_title,
            None,
            ns_string!(""),
        )
    };
    main_menu.addItem(&view_menu_item);

    let view_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &view_title);
    let theme_title = NSString::from_str(text("menu.theme"));
    let theme_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &theme_title,
            None,
            ns_string!(""),
        )
    };
    theme_item.setSubmenu(Some(&build_theme_menu(mtm, target)));
    view_menu.addItem(&theme_item);
    let language_title = NSString::from_str(text("menu.language"));
    let language_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &language_title,
            None,
            ns_string!(""),
        )
    };
    language_item.setSubmenu(Some(&build_language_menu(mtm, target)));
    view_menu.addItem(&language_item);
    let size_title = NSString::from_str(text("menu.size"));
    let size_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &size_title,
            None,
            ns_string!(""),
        )
    };
    size_item.setSubmenu(Some(&build_size_menu(mtm, target)));
    view_menu.addItem(&size_item);
    view_menu.addItem(&NSMenuItem::separatorItem(mtm));
    let outline_item = action(
        mtm,
        text("menu.outline"),
        Some(sel!(toggleOutlinePanel:)),
        "",
        NSEventModifierFlags::empty(),
        target,
    );
    outline_item.setEnabled(false);
    remember_outline_item(&outline_item);
    view_menu.addItem(&outline_item);
    view_menu.addItem(&NSMenuItem::separatorItem(mtm));
    view_menu.addItem(&action(
        mtm,
        text("menu.toggle_full_screen"),
        Some(sel!(toggleFullscreenWindow:)),
        "f",
        NSEventModifierFlags::Control | NSEventModifierFlags::Command,
        target,
    ));

    unsafe {
        let _: () = objc2::msg_send![&*view_menu, setDelegate: target];
    }
    view_menu_item.setSubmenu(Some(&view_menu));
}

fn build_size_menu(mtm: MainThreadMarker, target: &AnyObject) -> Retained<NSMenu> {
    let title = NSString::from_str(text("menu.size"));
    let size_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &title);
    for (label, selector) in [
        (text("menu.zoom_in"), sel!(increaseDocumentSize:)),
        (text("menu.zoom_out"), sel!(decreaseDocumentSize:)),
        (text("menu.reset"), sel!(resetDocumentSize:)),
    ] {
        size_menu.addItem(&action(
            mtm,
            label,
            Some(selector),
            "",
            NSEventModifierFlags::empty(),
            target,
        ));
    }
    size_menu
}

fn build_theme_menu(mtm: MainThreadMarker, target: &AnyObject) -> Retained<NSMenu> {
    let title = NSString::from_str(text("menu.theme"));
    let theme_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &title);

    for theme in AppTheme::ALL {
        let item = action(
            mtm,
            theme_label(theme),
            Some(selector_for_theme(theme)),
            "",
            NSEventModifierFlags::empty(),
            target,
        );
        remember_theme_item(slot_for_theme(theme), &item);
        theme_menu.addItem(&item);
    }

    theme_menu
}

fn build_language_menu(mtm: MainThreadMarker, target: &AnyObject) -> Retained<NSMenu> {
    let title = NSString::from_str(text("menu.language"));
    let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &title);
    for language in AppLanguage::ALL {
        let item = action(
            mtm,
            language_label(language),
            Some(selector_for_language(language)),
            "",
            NSEventModifierFlags::empty(),
            target,
        );
        remember_language_item(language_slot(language), &item);
        menu.addItem(&item);
    }
    menu
}

fn theme_label(theme: AppTheme) -> &'static str {
    match theme {
        AppTheme::Default => text("menu.theme_default"),
        AppTheme::Dark => text("menu.theme_dark"),
        AppTheme::Light => text("menu.theme_light"),
    }
}

fn slot_for_theme(theme: AppTheme) -> ThemeMenuSlot {
    match theme {
        AppTheme::Default => ThemeMenuSlot::Default,
        AppTheme::Dark => ThemeMenuSlot::Dark,
        AppTheme::Light => ThemeMenuSlot::Light,
    }
}

fn selector_for_theme(theme: AppTheme) -> objc2::runtime::Sel {
    match theme {
        AppTheme::Default => sel!(selectDefaultTheme:),
        AppTheme::Dark => sel!(selectDarkTheme:),
        AppTheme::Light => sel!(selectLightTheme:),
    }
}

fn language_slot(language: AppLanguage) -> LanguageMenuSlot {
    match language {
        AppLanguage::English => LanguageMenuSlot::English,
        AppLanguage::SimplifiedChinese => LanguageMenuSlot::SimplifiedChinese,
    }
}

fn language_label(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => text("language.english"),
        AppLanguage::SimplifiedChinese => text("language.simplified_chinese"),
    }
}

fn selector_for_language(language: AppLanguage) -> objc2::runtime::Sel {
    match language {
        AppLanguage::English => sel!(selectEnglishLanguage:),
        AppLanguage::SimplifiedChinese => sel!(selectSimplifiedChineseLanguage:),
    }
}
