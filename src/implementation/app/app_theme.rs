#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppTheme {
    Default,
    Dark,
}

impl AppTheme {
    #[cfg(test)]
    pub(crate) const ALL: [AppTheme; 2] = [AppTheme::Default, AppTheme::Dark];

    pub(crate) const fn key(self) -> &'static str {
        match self {
            AppTheme::Default => "default",
            AppTheme::Dark => "dark",
        }
    }

    #[cfg(test)]
    pub(crate) fn from_key(value: &str) -> Option<Self> {
        match value {
            "default" => Some(AppTheme::Default),
            "dark" => Some(AppTheme::Dark),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThemePreference {
    System,
    Default,
    Dark,
}

impl ThemePreference {
    pub(crate) const ALL: [Self; 3] = [Self::System, Self::Default, Self::Dark];

    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Default => "default",
            Self::Dark => "dark",
        }
    }

    pub(crate) fn from_stored_key(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "default" | "light" => Some(Self::Default),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    pub(crate) fn resolve(self, system_theme: tao::window::Theme) -> AppTheme {
        match self {
            Self::Default => AppTheme::Default,
            Self::Dark => AppTheme::Dark,
            Self::System => match system_theme {
                tao::window::Theme::Dark => AppTheme::Dark,
                _ => AppTheme::Default,
            },
        }
    }
}
