use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{NSString, ns_string};

use super::menu_file_items::action;
use crate::app::text;

pub(super) fn add_edit_menu(mtm: MainThreadMarker, main_menu: &NSMenu, target: &AnyObject) {
    let edit_title = NSString::from_str(text("menu.edit"));
    let edit_menu_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &edit_title,
            None,
            ns_string!(""),
        )
    };
    main_menu.addItem(&edit_menu_item);

    let edit_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &edit_title);
    edit_menu.addItem(&action(
        mtm,
        text("menu.toggle_mode"),
        Some(sel!(toggleDocumentMode:)),
        "/",
        NSEventModifierFlags::Command,
        target,
    ));
    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
    let undo_title = NSString::from_str(text("menu.undo"));
    let undo_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &undo_title,
            Some(sel!(undo:)),
            ns_string!("z"),
        )
    };
    undo_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    edit_menu.addItem(&undo_item);

    let redo_title = NSString::from_str(text("menu.redo"));
    let redo_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &redo_title,
            Some(sel!(redo:)),
            ns_string!("r"),
        )
    };
    redo_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    edit_menu.addItem(&redo_item);
    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
    let find_title = NSString::from_str(text("menu.find"));
    let find_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &find_title,
            Some(sel!(openFindPanel:)),
            ns_string!("f"),
        )
    };
    unsafe { find_item.setTarget(Some(target)) };
    find_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    edit_menu.addItem(&find_item);
    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
    edit_menu.addItem(&basic_system_item(
        mtm,
        text("menu.cut"),
        Some(sel!(cut:)),
        "x",
    ));
    edit_menu.addItem(&basic_system_item(
        mtm,
        text("menu.copy"),
        Some(sel!(copy:)),
        "c",
    ));
    edit_menu.addItem(&basic_system_item(
        mtm,
        text("menu.paste"),
        Some(sel!(paste:)),
        "v",
    ));
    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
    edit_menu.addItem(&basic_system_item(
        mtm,
        text("menu.select_all"),
        Some(sel!(selectAll:)),
        "a",
    ));
    edit_menu_item.setSubmenu(Some(&edit_menu));
}

fn basic_system_item(
    mtm: MainThreadMarker,
    title: &str,
    action: Option<objc2::runtime::Sel>,
    key: &str,
) -> objc2::rc::Retained<NSMenuItem> {
    let title = NSString::from_str(title);
    let key_value = NSString::from_str(key);
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &title,
            action,
            &key_value,
        )
    };
    item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    item
}
