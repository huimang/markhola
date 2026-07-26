use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{NSString, ns_string};

use crate::app::text;

use super::menu_file_export::build_export_menu;
use super::menu_file_items::action;
use super::menu_state::{remember_print, remember_save_as};

pub(super) fn add_file_menu(mtm: MainThreadMarker, main_menu: &NSMenu, target: &AnyObject) {
    let file_title = NSString::from_str(text("menu.file"));
    let file_menu_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &file_title,
            None,
            ns_string!(""),
        )
    };
    main_menu.addItem(&file_menu_item);

    let file_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &file_title);
    file_menu.addItem(&action(
        mtm,
        text("menu.new"),
        Some(sel!(newMenuDocument:)),
        "n",
        NSEventModifierFlags::Command,
        target,
    ));
    file_menu.addItem(&action(
        mtm,
        text("menu.open"),
        Some(sel!(openMenuDocument:)),
        "o",
        NSEventModifierFlags::Command,
        target,
    ));
    file_menu.addItem(&action(
        mtm,
        text("menu.save"),
        Some(sel!(saveMenuDocument:)),
        "s",
        NSEventModifierFlags::Command,
        target,
    ));

    let save_as = action(
        mtm,
        text("menu.save_as"),
        Some(sel!(saveMenuDocumentAs:)),
        "S",
        NSEventModifierFlags::Command | NSEventModifierFlags::Shift,
        target,
    );
    remember_save_as(&save_as);
    file_menu.addItem(&save_as);

    file_menu.addItem(&NSMenuItem::separatorItem(mtm));

    let print = action(
        mtm,
        text("menu.print"),
        Some(sel!(printDocument:)),
        "p",
        NSEventModifierFlags::Command,
        target,
    );
    print.setEnabled(false);
    remember_print(&print);
    file_menu.addItem(&print);

    let export_title = NSString::from_str(text("menu.export"));
    let export_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &export_title,
            None,
            ns_string!(""),
        )
    };
    file_menu.addItem(&export_item);
    export_item.setSubmenu(Some(&build_export_menu(mtm, target)));

    file_menu.addItem(&NSMenuItem::separatorItem(mtm));
    file_menu.addItem(&action(
        mtm,
        text("menu.close"),
        Some(sel!(closeCurrentDocument:)),
        "w",
        NSEventModifierFlags::Command,
        target,
    ));
    file_menu.addItem(&action(
        mtm,
        text("menu.exit"),
        Some(sel!(exitApplication:)),
        "q",
        NSEventModifierFlags::Command,
        target,
    ));

    file_menu_item.setSubmenu(Some(&file_menu));
}
