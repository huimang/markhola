#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DocumentSize {
    percent: u16,
}

impl DocumentSize {
    pub(crate) const DEFAULT_PERCENT: u16 = 100;
    pub(crate) const MIN_PERCENT: u16 = 50;
    pub(crate) const MAX_PERCENT: u16 = 200;
    pub(crate) const STEP_PERCENT: u16 = 10;

    pub(crate) const fn default_size() -> Self {
        Self {
            percent: Self::DEFAULT_PERCENT,
        }
    }

    pub(crate) fn from_stored(value: i64) -> Self {
        let Ok(percent) = u16::try_from(value) else {
            return Self::default_size();
        };
        if (Self::MIN_PERCENT..=Self::MAX_PERCENT).contains(&percent)
            && percent.is_multiple_of(Self::STEP_PERCENT)
        {
            Self { percent }
        } else {
            Self::default_size()
        }
    }

    pub(crate) const fn percent(self) -> u16 {
        self.percent
    }

    pub(crate) fn increase(self) -> Self {
        Self {
            percent: self
                .percent
                .saturating_add(Self::STEP_PERCENT)
                .min(Self::MAX_PERCENT),
        }
    }

    pub(crate) fn decrease(self) -> Self {
        Self {
            percent: self
                .percent
                .saturating_sub(Self::STEP_PERCENT)
                .max(Self::MIN_PERCENT),
        }
    }
}

impl Default for DocumentSize {
    fn default() -> Self {
        Self::default_size()
    }
}
