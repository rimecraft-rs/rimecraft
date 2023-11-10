use once_cell::sync::Lazy;

use std::{collections::HashMap, str::FromStr};

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct ClickEvent {
    action: ClickEventAction,
    value: String,
}

impl ClickEvent {
    #[inline]
    pub fn new(action: ClickEventAction, value: String) -> Self {
        Self { action, value }
    }

    #[inline]
    pub fn action(&self) -> ClickEventAction {
        self.action
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClickEventAction {
    OpenUrl,
    OpenFile,
    RunCommand,
    SuggestCommand,
    ChangePage,
    CopyToClipboard,
}

impl ClickEventAction {
    const VALUES: [Self; 6] = [
        Self::OpenUrl,
        Self::OpenFile,
        Self::RunCommand,
        Self::SuggestCommand,
        Self::ChangePage,
        Self::CopyToClipboard,
    ];

    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::OpenUrl => "open_url",
            Self::OpenFile => "open_file",
            Self::RunCommand => "run_command",
            Self::SuggestCommand => "suggest_command",
            Self::ChangePage => "change_page",
            Self::CopyToClipboard => "copy_to_clipboard",
        }
    }

    #[inline]
    pub fn is_user_definable(self) -> bool {
        !matches!(self, Self::OpenFile)
    }
}

impl FromStr for ClickEventAction {
    type Err = super::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static MAP: Lazy<HashMap<String, ClickEventAction>> = Lazy::new(|| {
            ClickEventAction::VALUES
                .into_iter()
                .map(|value| (value.name().to_owned(), value))
                .collect()
        });

        MAP.get(s)
            .copied()
            .ok_or_else(|| super::Error::InvalidName(s.to_owned()))
    }
}
