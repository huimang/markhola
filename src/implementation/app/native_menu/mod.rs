//! Native menu definitions, state, installation, and target/action code.

#[cfg(target_os = "macos")]
pub(crate) mod menu_app;
#[cfg(target_os = "macos")]
pub(crate) mod menu_edit;
#[cfg(target_os = "macos")]
pub(crate) mod menu_file;
#[cfg(target_os = "macos")]
pub(crate) mod menu_file_export;
#[cfg(target_os = "macos")]
pub(crate) mod menu_file_items;
#[cfg(target_os = "macos")]
pub(crate) mod menu_help;
#[cfg(target_os = "macos")]
pub(crate) mod menu_install;
#[cfg(target_os = "macos")]
pub(crate) mod menu_state;
#[cfg(target_os = "macos")]
pub(crate) mod menu_tab;
#[cfg(target_os = "macos")]
pub(crate) mod menu_target;

#[cfg(target_os = "macos")]
pub(crate) mod macos_menu;

#[cfg(target_os = "windows")]
pub(crate) mod windows_menu;
