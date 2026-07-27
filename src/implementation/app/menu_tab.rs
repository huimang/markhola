use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{NSString, ns_string};

use crate::app::text;

pub(super) fn add_tab_menu(mtm: MainThreadMarker, main_menu: &NSMenu, target: &AnyObject) {
    let tab_title = NSString::from_str(text("menu.tab"));
    let tab_menu_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &tab_title,
            None,
            ns_string!(""),
        )
    };
    main_menu.addItem(&tab_menu_item);

    let tab_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &tab_title);
    tab_menu.addItem(&action(
        mtm,
        text("menu.next_tab"),
        Some(sel!(activateNextDocument:)),
        "]",
        NSEventModifierFlags::Command | NSEventModifierFlags::Shift,
        target,
    ));
    tab_menu.addItem(&action(
        mtm,
        text("menu.previous_tab"),
        Some(sel!(activatePreviousDocument:)),
        "[",
        NSEventModifierFlags::Command | NSEventModifierFlags::Shift,
        target,
    ));
    tab_menu.addItem(&NSMenuItem::separatorItem(mtm));
    tab_menu.addItem(&action(
        mtm,
        text("menu.close_tab"),
        Some(sel!(closeCurrentDocument:)),
        "w",
        NSEventModifierFlags::Command,
        target,
    ));
    tab_menu.addItem(&action(
        mtm,
        text("menu.close_other_tabs"),
        Some(sel!(closeOtherDocuments:)),
        "w",
        NSEventModifierFlags::Command | NSEventModifierFlags::Option,
        target,
    ));
    tab_menu.addItem(&action(
        mtm,
        text("menu.close_all_tabs"),
        Some(sel!(closeAllDocuments:)),
        "w",
        NSEventModifierFlags::Command | NSEventModifierFlags::Option | NSEventModifierFlags::Shift,
        target,
    ));
    tab_menu_item.setSubmenu(Some(&tab_menu));
}

fn action(
    mtm: MainThreadMarker,
    title: &str,
    action: Option<objc2::runtime::Sel>,
    key: &str,
    modifiers: NSEventModifierFlags,
    target: &AnyObject,
) -> objc2::rc::Retained<NSMenuItem> {
    let title = NSString::from_str(title);
    let key = NSString::from_str(key);
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &title, action, &key)
    };
    unsafe { item.setTarget(Some(target)) };
    item.setKeyEquivalentModifierMask(modifiers);
    item
}
