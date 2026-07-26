#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppTheme {
    Default,
    Dark,
    Light,
}

impl AppTheme {
    pub(crate) const ALL: [AppTheme; 3] = [AppTheme::Default, AppTheme::Dark, AppTheme::Light];

    pub(crate) const fn key(self) -> &'static str {
        match self {
            AppTheme::Default => "default",
            AppTheme::Dark => "dark",
            AppTheme::Light => "light",
        }
    }

    pub(crate) fn from_key(value: &str) -> Option<Self> {
        match value {
            "default" => Some(AppTheme::Default),
            "dark" => Some(AppTheme::Dark),
            "light" => Some(AppTheme::Light),
            _ => None,
        }
    }
}
