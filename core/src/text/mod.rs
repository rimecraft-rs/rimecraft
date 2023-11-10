use std::{
    borrow::Cow,
    fmt::Display,
    hash::{Hash, Hasher},
    str::FromStr,
};

use rimecraft_primitives::{id, Id};

use crate::Rgb;

use self::{click_event::ClickEvent, hover_event::HoverEvent};

use super::formatting::Formatting;

pub mod click_event;
pub mod hover_event;

pub mod visit;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not a valid color")]
    InvalidColor,
    #[error("no valid color value found")]
    ColorValueNotFound,
    #[error("unable to parse integer: {0}")]
    ParseInt(std::num::ParseIntError),
    #[error("formatting error: {0}")]
    Formatting(super::formatting::Error),
    #[error("invalid name: {0}")]
    InvalidName(String),
}

/// TODO: Implement net.minecraft.text.Text
pub trait Text {
    fn style(&self) -> &Style;
    fn siblings(&self) -> Vec<Box<dyn Text>>;
    /// TODO: Implement net.minecraft.text.OrderedText
    fn as_ordered_text(&self) -> ();
}

/// The style of a [`Text`].\
/// A style is immutable.
#[derive(PartialEq)]
pub struct Style {
    pub color: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub click: Option<ClickEvent>,
    pub hover: Option<HoverEvent>,
    pub insertion: Option<String>,
    pub font: Option<Id>,
}

impl Style {
    const EMPTY: Self = Self {
        color: None,
        bold: None,
        italic: None,
        underlined: None,
        strikethrough: None,
        obfuscated: None,
        click: None,
        hover: None,
        insertion: None,
        font: None,
    };

    const DEFAULT_FONT_ID: &str = "default";

    #[inline]
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    #[inline]
    pub fn bold(&self) -> bool {
        self.bold.unwrap_or(false)
    }

    #[inline]
    pub fn italic(&self) -> bool {
        self.italic.unwrap_or(false)
    }

    #[inline]
    pub fn strikethrough(&self) -> bool {
        self.strikethrough.unwrap_or(false)
    }

    #[inline]
    pub fn underlined(&self) -> bool {
        self.underlined.unwrap_or(false)
    }

    #[inline]
    pub fn obfuscated(&self) -> bool {
        self.obfuscated.unwrap_or(false)
    }

    pub fn empty(&self) -> bool {
        self == &Self::EMPTY
    }

    pub fn click(&self) -> Option<&ClickEvent> {
        self.click.as_ref()
    }

    pub fn hover(&self) -> Option<&HoverEvent> {
        self.hover.as_ref()
    }

    pub fn insertion(&self) -> Option<&String> {
        self.insertion.as_ref()
    }

    pub fn font(&self) -> Cow<'_, Id> {
        self.font
            .as_ref()
            .map_or_else(|| Cow::Owned(id!(Self::DEFAULT_FONT_ID)), Cow::Borrowed)
    }
}

/// Represents an RGB color of a [`Text`].
///
/// This is immutable as a part of [`Style`].
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.TextColor` (yarn).
#[derive(Debug, Eq)]
pub struct Color {
    /// A 24-bit color.
    rgb: Rgb,
    name: Option<&'static str>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    #[inline]
    pub fn rgb(&self) -> Rgb {
        self.rgb
    }

    #[inline]
    fn to_hex_code(&self) -> String {
        format!("{}{:06X}", Self::RGB_PREFIX, self.rgb)
    }

    pub fn name(&self) -> Cow<'static, str> {
        self.name
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(self.to_hex_code()))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name {
            f.write_str(name)
        } else {
            f.write_str(&self.to_hex_code())
        }
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(value) = s.strip_prefix(Self::RGB_PREFIX) {
            Ok(Self {
                rgb: value.parse().map_err(Error::ParseInt)?,
                name: None,
            })
        } else {
            s.parse::<Formatting>()
                .map_err(Error::Formatting)?
                .try_into()
        }
    }
}

impl TryFrom<Formatting> for Color {
    type Error = Error;

    fn try_from(value: Formatting) -> Result<Self, Self::Error> {
        Ok(Self {
            rgb: value.color_value().ok_or(Error::ColorValueNotFound)?,
            name: Some(value.name()),
        })
    }
}

impl PartialEq for Color {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.rgb == other.rgb
    }
}

impl Hash for Color {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rgb.hash(state);
        self.name.hash(state)
    }
}
