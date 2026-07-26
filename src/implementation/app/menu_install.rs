use std::error::Error;

use objc2::{MainThreadOnly, sel};
use objc2_app_kit::{NSApp, NSApplication, NSMenu};
use objc2_foundation::MainThreadMarker;
use objc2_foundation::ns_string;
use tao::event_loop::EventLoopProxy;

use crate::app::UserEvent;

use super::menu_app::add_app_menu;
use super::menu_edit::add_edit_menu;
use super::menu_file::add_file_menu;
use super::menu_help::add_help_menu;
use super::menu_state::set_selected_language;
use super::menu_tab::add_tab_menu;
use super::menu_target::target_ref;
use super::menu_view::add_view_menu;
use crate::app::current_language;

pub fn install(proxy: &EventLoopProxy<UserEvent>) -> Result<(), Box<dyn Error>> {
    let mtm = MainThreadMarker::new().ok_or("menu setup must run on main thread")?;
    let app = NSApplication::sharedApplication(mtm);
    let target = target_ref(mtm, proxy.clone());
    let main_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!("MainMenu"));

    add_app_menu(mtm, &main_menu, target);
    add_file_menu(mtm, &main_menu, target);
    add_edit_menu(mtm, &main_menu, target);
    add_tab_menu(mtm, &main_menu, target);
    add_view_menu(mtm, &main_menu, target);
    add_help_menu(mtm, &main_menu, target);

    app.setMainMenu(Some(&main_menu));
    remove_window_tab_items(&main_menu);
    set_selected_language(current_language());
    let _ = NSApp(mtm);
    Ok(())
}

pub fn remove_window_tab_items_from_main_menu() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    if let Some(main_menu) = app.mainMenu() {
        remove_window_tab_items(&main_menu);
    }
}

pub(super) fn remove_window_tab_items(menu: &NSMenu) {
    for index in (0..menu.numberOfItems()).rev() {
        let Some(item) = menu.itemAtIndex(index) else {
            continue;
        };
        if matches!(
            item.action(),
            Some(action) if action == sel!(toggleTabBar:) || action == sel!(toggleTabOverview:)
        ) {
            menu.removeItemAtIndex(index);
            continue;
        }
        if let Some(submenu) = item.submenu() {
            remove_window_tab_items(&submenu);
        }
    }
    while menu
        .itemAtIndex(0)
        .is_some_and(|item| item.isSeparatorItem())
    {
        menu.removeItemAtIndex(0);
    }
    while menu
        .numberOfItems()
        .checked_sub(1)
        .and_then(|index| menu.itemAtIndex(index))
        .is_some_and(|item| item.isSeparatorItem())
    {
        menu.removeItemAtIndex(menu.numberOfItems() - 1);
    }
}
