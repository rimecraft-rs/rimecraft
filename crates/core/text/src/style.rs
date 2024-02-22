//! Text style related types.

use std::{
    borrow::Cow,
    fmt::Display,
    hash::Hash,
    ops::{Add, AddAssign},
    str::FromStr,
};

use rgb::RGB8;
use rimecraft_fmt::Formatting;

use crate::Error;

/// An RGB color of a text.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    rgb: RGB8,
    name: Option<&'static str>,
}

impl Color {
    /// Gets RGB value of the color.
    #[inline]
    pub const fn rgb(&self) -> RGB8 {
        self.rgb
    }

    #[inline]
    const fn to_u24(self) -> u32 {
        u32::from_be_bytes([0x00, self.rgb.r, self.rgb.g, self.rgb.b])
    }

    /// Gets the name of the color.
    #[inline]
    pub fn name(&self) -> Cow<'static, str> {
        match self.name {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(format!("#{:06X}", self.to_u24())),
        }
    }
}

impl PartialEq for Color {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.rgb == other.rgb
    }
}

impl Eq for Color {}

impl Hash for Color {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rgb.hash(state);
    }
}

impl Display for Color {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name().as_ref())
    }
}

impl From<RGB8> for Color {
    #[inline]
    fn from(rgb: RGB8) -> Self {
        Self { rgb, name: None }
    }
}

impl TryFrom<Formatting> for Color {
    type Error = Error;

    #[inline]
    fn try_from(value: Formatting) -> Result<Self, Self::Error> {
        value
            .color_value()
            .map(|rgb| Self {
                rgb,
                name: Some(value.name()),
            })
            .ok_or(Error::FormattingWithoutColor(value))
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(hex) = s.strip_prefix('#') {
            let rgb =
                u32::from_str_radix(hex, 16).map_err(|_| Error::InvalidColor(s.to_owned()))?;
            if rgb > 0x00FF_FFFF {
                return Err(Error::ColorValueOutOfRange(s.to_owned()));
            }
            let [_, r, g, b] = rgb.to_be_bytes();
            Ok(Self {
                rgb: RGB8::new(r, g, b),
                name: None,
            })
        } else {
            s.parse::<Formatting>()?.try_into()
        }
    }
}

/// Style of a text, representing cosmetic attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Style<Ext> {
    /// Color of the text.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub color: Option<Color>,

    /// Whether the text is bold.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub bold: Option<bool>,

    /// Whether the text is italic.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub italic: Option<bool>,

    /// Whether the text is underlined.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub underlined: Option<bool>,

    /// Whether the text is strikethrough.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub strikethrough: Option<bool>,

    /// Whether the text is obfuscated.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub obfuscated: Option<bool>,

    /// Extra data.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub ext: Ext,
}

impl<Ext> Add for Style<Ext>
where
    Ext: Add<Output = Ext>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            color: rhs.color.or(self.color),
            bold: rhs.bold.or(self.bold),
            italic: rhs.italic.or(self.italic),
            underlined: rhs.underlined.or(self.underlined),
            strikethrough: rhs.strikethrough.or(self.strikethrough),
            obfuscated: rhs.obfuscated.or(self.obfuscated),
            ext: self.ext + rhs.ext,
        }
    }
}

impl<Ext> AddAssign for Style<Ext>
where
    Ext: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.color = rhs.color.or(self.color);
        self.bold = rhs.bold.or(self.bold);
        self.italic = rhs.italic.or(self.italic);
        self.underlined = rhs.underlined.or(self.underlined);
        self.strikethrough = rhs.strikethrough.or(self.strikethrough);
        self.obfuscated = rhs.obfuscated.or(self.obfuscated);
        self.ext += rhs.ext;
    }
}

#[cfg(feature = "serde")]
mod _serde {
    use super::*;

    use serde::{Deserialize, Serialize};

    impl Serialize for Color {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.name().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Color {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            <&str>::deserialize(deserializer)?
                .parse()
                .map_err(serde::de::Error::custom)
        }
    }
}
