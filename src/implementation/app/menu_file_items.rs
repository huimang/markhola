use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSEventModifierFlags, NSMenuItem};
use objc2_foundation::NSString;

pub(super) fn action(
    mtm: MainThreadMarker,
    title: &str,
    action: Option<objc2::runtime::Sel>,
    key: &str,
    modifiers: NSEventModifierFlags,
    target: &AnyObject,
) -> objc2::rc::Retained<NSMenuItem> {
    let item = item_with_key(mtm, title, action, key);
    unsafe { item.setTarget(Some(target)) };
    item.setKeyEquivalentModifierMask(modifiers);
    item
}

fn item_with_key(
    mtm: MainThreadMarker,
    title: &str,
    action: Option<objc2::runtime::Sel>,
    key: &str,
) -> objc2::rc::Retained<NSMenuItem> {
    let title = NSString::from_str(title);
    let key = NSString::from_str(key);
    unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &title, action, &key)
    }
}
