use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, sel};
use objc2_app_kit::{NSMenu, NSMenuItem};
use objc2_foundation::{NSString, ns_string};

use crate::app::text;

pub(super) fn add_help_menu(mtm: MainThreadMarker, main_menu: &NSMenu, target: &AnyObject) {
    let help_title = NSString::from_str(text("menu.help"));
    let help_menu_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &help_title,
            None,
            ns_string!(""),
        )
    };
    main_menu.addItem(&help_menu_item);

    let help_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &help_title);
    let documentation_title = NSString::from_str(text("menu.documentation"));
    let documentation_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &documentation_title,
            Some(sel!(openDocumentation:)),
            ns_string!(""),
        )
    };
    unsafe { documentation_item.setTarget(Some(target)) };
    help_menu.addItem(&documentation_item);
    help_menu_item.setSubmenu(Some(&help_menu));
}
