use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{NSString, ns_string};

use crate::app::text;

pub(super) fn add_app_menu(mtm: MainThreadMarker, main_menu: &NSMenu, target: &AnyObject) {
    let app_title = NSString::from_str("MarkHola");
    let app_menu_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &app_title,
            None,
            ns_string!(""),
        )
    };
    main_menu.addItem(&app_menu_item);

    let app_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &app_title);
    let about_title = NSString::from_str(text("menu.about"));
    let about_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &about_title,
            Some(sel!(showAboutPanel:)),
            ns_string!(""),
        )
    };
    unsafe { about_item.setTarget(Some(target)) };
    let exit_title = NSString::from_str(text("menu.exit"));
    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &exit_title,
            Some(sel!(exitApplication:)),
            ns_string!("q"),
        )
    };
    unsafe { quit_item.setTarget(Some(target)) };
    quit_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    app_menu.addItem(&about_item);
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    app_menu.addItem(&quit_item);
    app_menu_item.setSubmenu(Some(&app_menu));
}
