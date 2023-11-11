use std::{
    borrow::Cow,
    fmt::Display,
    hash::{Hash, Hasher},
    str::FromStr,
};

use rimecraft_primitives::{id, Id};

use crate::RGB;

use super::fmt::Formatting;

pub mod click_event;
pub mod hover_event;

pub mod visit;

pub use click_event::ClickEvent;
pub use hover_event::HoverEvent;

/// An error that can occur when processing a [`Text`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not a valid color")]
    InvalidColor,
    #[error("no valid color value found")]
    ColorValueNotFound,
    #[error("unable to parse integer: {0}")]
    ParseInt(std::num::ParseIntError),
    #[error("formatting error: {0}")]
    Formatting(super::fmt::Error),
    #[error("invalid name: {0}")]
    InvalidName(String),
}

//TODO: Implement net.minecraft.text.Text
pub trait Text {
    fn style(&self) -> &Style;
    fn siblings(&self) -> Vec<Box<dyn Text>>;
    //TODO: Implement net.minecraft.text.OrderedText
    fn as_ordered_text(&self);
}

/// The style of a [`Text`], representing cosmetic attributes.
/// It includes font, color, click event, hover event, etc.
///
/// A style should be immutable.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.Style` (yarn).
#[derive(PartialEq, Debug, Default)]
pub struct Style {
    /// The color of this style.
    pub color: Option<Color>,

    /// Whether this style has bold formatting.
    pub bold: Option<bool>,

    /// Whether this style has italic formatting.
    pub italic: Option<bool>,

    /// Whether this style has underlined formatting.
    pub underlined: Option<bool>,

    /// Whether this style has strikethrough formatting.
    pub strikethrough: Option<bool>,

    /// Whether this style has obfuscated formatting.
    pub obfuscated: Option<bool>,

    /// The click event of this style.
    pub click: Option<ClickEvent>,

    /// The hover event of this style.
    pub hover: Option<HoverEvent>,

    /// The insertion text of this style.
    pub insertion: Option<String>,

    /// The font ID of this style.
    pub font: Option<Id>,
}

impl Style {
    /// An empty style.
    pub const EMPTY: Self = Self {
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

    /// Returns the color of this style.
    #[inline]
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// Returns whether this style has bold formatting.
    ///
    /// See [`Formatting::Bold`].
    #[inline]
    pub fn is_bold(&self) -> bool {
        self.bold.unwrap_or(false)
    }

    /// Returns whether this style has italic formatting.
    ///
    /// See [`Formatting::Italic`].
    #[inline]
    pub fn is_italic(&self) -> bool {
        self.italic.unwrap_or(false)
    }

    /// Returns whether this style has strikethrough formatting.
    ///
    /// See [`Formatting::Strikethrough`].
    #[inline]
    pub fn is_strikethrough(&self) -> bool {
        self.strikethrough.unwrap_or(false)
    }

    /// Returns whether this style has underlined formatting.
    ///
    /// See [`Formatting::Underline`].
    #[inline]
    pub fn is_underlined(&self) -> bool {
        self.underlined.unwrap_or(false)
    }

    /// Returns whether this style has obfuscated formatting.
    ///
    /// See [`Formatting::Obfuscated`].
    #[inline]
    pub fn is_obfuscated(&self) -> bool {
        self.obfuscated.unwrap_or(false)
    }

    /// Returns whether this style is empty.
    ///
    /// See [`Self::EMPTY`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self == &Self::EMPTY
    }

    /// Returns the click event of this style.
    #[inline]
    pub fn click_event(&self) -> Option<&ClickEvent> {
        self.click.as_ref()
    }

    /// Returns the hover event of this style.
    #[inline]
    pub fn hover_event(&self) -> Option<&HoverEvent> {
        self.hover.as_ref()
    }

    /// Returns the insertion text of this style.
    ///
    /// An insertion text is a text that is inserted into the chat
    /// when the player shift-clicks on the text.
    #[inline]
    pub fn insertion(&self) -> Option<&String> {
        self.insertion.as_ref()
    }

    /// Returns the font ID of this style.
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
    rgb: RGB,
    name: Option<&'static str>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    /// Returns the inner RGB value of this color.
    #[inline]
    pub fn rgb(&self) -> RGB {
        self.rgb
    }

    /// Returns the hex code of this color.
    #[inline]
    fn to_hex_code(&self) -> String {
        format!("{}{:06X}", Self::RGB_PREFIX, self.rgb)
    }

    /// Returns the name of this color.
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
