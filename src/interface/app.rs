#[path = "../implementation/app/implementation.rs"]
mod implementation;
#[path = "../implementation/app/interface.rs"]
mod interface;
pub(crate) use self::implementation::platform;
#[path = "../implementation/app/web_surface/shell.rs"]
mod shell;
#[cfg(test)]
#[path = "../tests/app.rs"]
mod tests;

pub use self::implementation::run;
pub(crate) use self::interface::*;
