use std::cell::RefCell;

use objc2::rc::Retained;
use objc2_app_kit::{NSControlStateValueOff, NSControlStateValueOn, NSMenuItem};

use crate::app::{AppLanguage, AppTheme};

thread_local! {
    static EXPORT_PDF_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static EXPORT_HTML_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static SAVE_AS_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static PRINT_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static DEFAULT_THEME_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static DARK_THEME_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static LIGHT_THEME_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static ENGLISH_LANGUAGE_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static CHINESE_LANGUAGE_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static OUTLINE_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
pub(super) enum ThemeMenuSlot {
    Default,
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub(super) enum LanguageMenuSlot {
    English,
    SimplifiedChinese,
}

pub(super) fn remember_save_as(item: &Retained<NSMenuItem>) {
    SAVE_AS_ITEM.with(|slot| *slot.borrow_mut() = Some(item.clone()));
}

pub(super) fn remember_print(item: &Retained<NSMenuItem>) {
    PRINT_ITEM.with(|slot| *slot.borrow_mut() = Some(item.clone()));
}

pub(super) fn remember_export_pdf(item: &Retained<NSMenuItem>) {
    EXPORT_PDF_ITEM.with(|slot| *slot.borrow_mut() = Some(item.clone()));
}

pub(super) fn remember_export_html(item: &Retained<NSMenuItem>) {
    EXPORT_HTML_ITEM.with(|slot| *slot.borrow_mut() = Some(item.clone()));
}

pub(super) fn remember_theme_item(slot: ThemeMenuSlot, item: &Retained<NSMenuItem>) {
    theme_item_slot(slot).with(|state| *state.borrow_mut() = Some(item.clone()));
}

pub(super) fn remember_language_item(slot: LanguageMenuSlot, item: &Retained<NSMenuItem>) {
    language_item_slot(slot).with(|state| *state.borrow_mut() = Some(item.clone()));
}

pub(super) fn remember_outline_item(item: &Retained<NSMenuItem>) {
    OUTLINE_ITEM.with(|slot| *slot.borrow_mut() = Some(item.clone()));
}

pub fn set_document_output_enabled(enabled: bool) {
    for_each_output_item(|item| item.setEnabled(enabled));
}

pub fn set_selected_theme(theme: AppTheme) {
    for slot in [
        ThemeMenuSlot::Default,
        ThemeMenuSlot::Dark,
        ThemeMenuSlot::Light,
    ] {
        theme_item_slot(slot).with(|state| {
            if let Some(item) = state.borrow().as_deref() {
                item.setState(if theme_for_slot(slot) == theme {
                    NSControlStateValueOn
                } else {
                    NSControlStateValueOff
                });
            }
        });
    }
}

pub fn set_selected_language(language: AppLanguage) {
    for slot in [
        LanguageMenuSlot::English,
        LanguageMenuSlot::SimplifiedChinese,
    ] {
        language_item_slot(slot).with(|state| {
            if let Some(item) = state.borrow().as_deref() {
                item.setState(if language_for_slot(slot) == language {
                    NSControlStateValueOn
                } else {
                    NSControlStateValueOff
                });
            }
        });
    }
}

pub fn set_outline_available(available: bool) {
    OUTLINE_ITEM.with(|slot| {
        if let Some(item) = slot.borrow().as_deref() {
            item.setEnabled(available);
            if !available {
                item.setState(NSControlStateValueOff);
            }
        }
    });
}

pub fn toggle_outline_selected() -> bool {
    OUTLINE_ITEM.with(|slot| {
        let item = slot.borrow();
        let Some(item) = item.as_deref() else {
            return false;
        };
        if !item.isEnabled() {
            item.setState(NSControlStateValueOff);
            return false;
        }
        let selected = item.state() != NSControlStateValueOn;
        item.setState(if selected {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        selected
    })
}

pub fn outline_selected() -> bool {
    OUTLINE_ITEM.with(|slot| {
        slot.borrow()
            .as_deref()
            .is_some_and(|item| item.state() == NSControlStateValueOn)
    })
}

pub fn set_outline_selected(selected: bool) {
    OUTLINE_ITEM.with(|slot| {
        if let Some(item) = slot.borrow().as_deref() {
            item.setState(if selected {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    });
}

fn for_each_output_item(mut f: impl FnMut(&NSMenuItem)) {
    SAVE_AS_ITEM.with(|slot| slot.borrow().as_deref().map(&mut f));
    PRINT_ITEM.with(|slot| slot.borrow().as_deref().map(&mut f));
    EXPORT_PDF_ITEM.with(|slot| slot.borrow().as_deref().map(&mut f));
    EXPORT_HTML_ITEM.with(|slot| slot.borrow().as_deref().map(&mut f));
}

fn theme_item_slot(
    slot: ThemeMenuSlot,
) -> &'static std::thread::LocalKey<RefCell<Option<Retained<NSMenuItem>>>> {
    match slot {
        ThemeMenuSlot::Default => &DEFAULT_THEME_ITEM,
        ThemeMenuSlot::Dark => &DARK_THEME_ITEM,
        ThemeMenuSlot::Light => &LIGHT_THEME_ITEM,
    }
}

fn theme_for_slot(slot: ThemeMenuSlot) -> AppTheme {
    match slot {
        ThemeMenuSlot::Default => AppTheme::Default,
        ThemeMenuSlot::Dark => AppTheme::Dark,
        ThemeMenuSlot::Light => AppTheme::Light,
    }
}

fn language_item_slot(
    slot: LanguageMenuSlot,
) -> &'static std::thread::LocalKey<RefCell<Option<Retained<NSMenuItem>>>> {
    match slot {
        LanguageMenuSlot::English => &ENGLISH_LANGUAGE_ITEM,
        LanguageMenuSlot::SimplifiedChinese => &CHINESE_LANGUAGE_ITEM,
    }
}

fn language_for_slot(slot: LanguageMenuSlot) -> AppLanguage {
    match slot {
        LanguageMenuSlot::English => AppLanguage::English,
        LanguageMenuSlot::SimplifiedChinese => AppLanguage::SimplifiedChinese,
    }
}
