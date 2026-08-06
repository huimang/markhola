#[path = "../implementation/app/app_theme.rs"]
mod app_theme;
#[path = "../implementation/app/implementation.rs"]
mod implementation;
#[path = "../implementation/app/interface.rs"]
mod interface;
#[cfg(target_os = "macos")]
#[path = "../implementation/app/macos_menu.rs"]
mod macos_menu;
#[cfg(test)]
#[path = "../tests/app.rs"]
mod tests;
#[cfg(test)]
#[path = "../tests/visual_package.rs"]
mod visual_package_tests;
#[path = "../implementation/app/web_localization.rs"]
mod web_localization;
#[path = "../implementation/app/web_surface/shell.rs"]
mod web_surface;

pub(crate) use self::app_theme::{AppTheme, ThemePreference};
pub use self::implementation::run;
pub(crate) use self::interface::*;
pub(crate) use self::web_localization::WebStrings;
