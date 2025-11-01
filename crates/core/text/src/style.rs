//! Text style related types.

use std::{
    borrow::Cow,
    fmt::Display,
    hash::Hash,
    ops::{Add, AddAssign},
    str::FromStr,
};

use remap::{remap, remap_method};
use rgb::{RGB8, alt::ARGB8};

use crate::Error;

pub use rimecraft_fmt::Formatting;

/// An RGB color of a text.
#[remap(yarn = "TextColor", mojmaps = "TextColor")]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    rgb: RGB8,
    name: Option<&'static str>,
}

impl Color {
    /// Gets RGB value of the color.
    #[inline]
    #[remap_method(yarn = "getRgb", mojmaps = "getValue")]
    pub const fn rgb(&self) -> RGB8 {
        self.rgb
    }

    #[inline]
    const fn to_u24(self) -> u32 {
        u32::from_be_bytes([0x00, self.rgb.r, self.rgb.g, self.rgb.b])
    }

    /// Gets the name of the color.
    #[inline]
    #[remap_method(yarn = "getName", mojmaps = "serialize")]
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

/// A shadow color, wrapping an [`ARGB8`] inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadowColor {
    argb: ARGB8,
}

impl ShadowColor {
    /// Creates a new shadow color from an [`ARGB8`].
    #[inline]
    pub const fn new(argb: ARGB8) -> Self {
        Self { argb }
    }

    /// Gets the inner [`ARGB8`] of this shadow color.
    #[inline]
    pub const fn into_inner(self) -> ARGB8 {
        self.argb
    }
}

impl From<ARGB8> for ShadowColor {
    #[inline]
    fn from(argb: ARGB8) -> Self {
        Self::new(argb)
    }
}

impl From<ShadowColor> for ARGB8 {
    #[inline]
    fn from(color: ShadowColor) -> Self {
        color.argb
    }
}

impl From<u32> for ShadowColor {
    #[inline]
    fn from(value: u32) -> Self {
        Self::new(ARGB8::from(value.to_be_bytes()))
    }
}

impl From<ShadowColor> for u32 {
    #[inline]
    fn from(color: ShadowColor) -> Self {
        Self::from_be_bytes(color.argb.into())
    }
}

/// A formattable type that can have [`Formatting`] applied to it.
///
/// All formatting operations take ownership of `self` and return a new instance with the formatting applied.
pub trait Formattable: Sized {
    /// Returns a new instance with the formatting applied.
    #[remap_method(yarn = "withFormatting", mojmaps = "applyFormat")]
    fn with_formatting(self, formatting: Formatting) -> Self {
        let _ = formatting;
        self
    }

    /// Returns a new instance with the formatting applied exclusively.
    #[remap_method(yarn = "withExclusiveFormatting", mojmaps = "applyLegacyFormat")]
    fn with_exclusive_formatting(self, formatting: Formatting) -> Self {
        let _ = formatting;
        self
    }
}

/// Style of a text, representing cosmetic attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[non_exhaustive]
#[remap(yarn = "Style", mojmaps = "Style")]
pub struct Style<Ext> {
    /// Color of the text.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub color: Option<Color>,

    /// Shadow color of the text.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none"),
        serde(default)
    )]
    pub shadow_color: Option<ShadowColor>,

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
            shadow_color: rhs.shadow_color.or(self.shadow_color),
            bold: rhs.bold.or(self.bold),
            italic: rhs.italic.or(self.italic),
            underlined: rhs.underlined.or(self.underlined),
            strikethrough: rhs.strikethrough.or(self.strikethrough),
            obfuscated: rhs.obfuscated.or(self.obfuscated),
            ext: self.ext + rhs.ext,
        }
    }
}

impl<'a, Ext> Add<&'a Self> for Style<Ext>
where
    Ext: Add<&'a Ext, Output = Ext>,
{
    type Output = Self;

    fn add(self, rhs: &'a Self) -> Self::Output {
        Self {
            color: rhs.color.or(self.color),
            shadow_color: rhs.shadow_color.or(self.shadow_color),
            bold: rhs.bold.or(self.bold),
            italic: rhs.italic.or(self.italic),
            underlined: rhs.underlined.or(self.underlined),
            strikethrough: rhs.strikethrough.or(self.strikethrough),
            obfuscated: rhs.obfuscated.or(self.obfuscated),
            ext: self.ext + &rhs.ext,
        }
    }
}

impl<Ext> AddAssign for Style<Ext>
where
    Ext: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.color = rhs.color.or(self.color);
        self.shadow_color = rhs.shadow_color.or(self.shadow_color);
        self.bold = rhs.bold.or(self.bold);
        self.italic = rhs.italic.or(self.italic);
        self.underlined = rhs.underlined.or(self.underlined);
        self.strikethrough = rhs.strikethrough.or(self.strikethrough);
        self.obfuscated = rhs.obfuscated.or(self.obfuscated);
        self.ext += rhs.ext;
    }
}

impl<Ext> Style<Ext> {
    /// Creates a new [`Style`] with the given extra data, with all other attributes unset.
    #[inline]
    pub fn new(ext: Ext) -> Self {
        Self {
            color: None,
            shadow_color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            ext,
        }
    }
}

impl<Ext> Formattable for Style<Ext>
where
    Ext: Formattable,
{
    /// Returns a new [`Style`] with the formatting provided and all other attributes of this style.
    fn with_formatting(self, formatting: Formatting) -> Self {
        let mut style = self;
        style.ext = style.ext.with_formatting(formatting);
        match formatting {
            Formatting::Bold => style.bold = Some(true),
            Formatting::Italic => style.italic = Some(true),
            Formatting::Underline => style.underlined = Some(true),
            Formatting::Strikethrough => style.strikethrough = Some(true),
            Formatting::Obfuscated => style.obfuscated = Some(true),
            Formatting::Reset => {
                // Reset clears everything
                return Self::new(style.ext);
            }
            _ => {
                style.color = formatting.try_into().ok();
            }
        }
        style
    }

    /// Returns a new [`Style`] with the formatting provided and some applicable attributes of this style.
    /// When a color formatting is passed for formatting, the other formattings, including bold, italic, strikethrough, underlined, and obfuscated, are all removed.
    fn with_exclusive_formatting(self, formatting: Formatting) -> Self {
        if formatting.is_color() {
            // Color formatting clears all modifiers
            Self {
                color: formatting.try_into().ok(),
                ..Self::new(self.ext.with_formatting(formatting))
            }
        } else {
            // Modifier formatting
            let mut style = self;
            style.ext = style.ext.with_formatting(formatting);
            match formatting {
                Formatting::Bold => style.bold = Some(true),
                Formatting::Italic => style.italic = Some(true),
                Formatting::Underline => style.underlined = Some(true),
                Formatting::Strikethrough => style.strikethrough = Some(true),
                Formatting::Obfuscated => style.obfuscated = Some(true),
                Formatting::Reset => {
                    // Reset clears everything
                    return Self::new(style.ext);
                }
                _ => {
                    style.color = formatting.try_into().ok();
                }
            }
            style
        }
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
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl serde::de::Visitor<'_> for Visitor {
                type Value = Color;

                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("a string")
                }

                #[inline]
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    v.parse().map_err(serde::de::Error::custom)
                }
            }

            deserializer.deserialize_str(Visitor)
        }
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    #[allow(variant_size_differences)]
    enum ARGB8Repr {
        Int(u32),
        Vec4f([f32; 4]),
    }

    impl Serialize for ShadowColor {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            u32::from(*self).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for ShadowColor {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(match ARGB8Repr::deserialize(deserializer)? {
                ARGB8Repr::Int(i) => i,
                ARGB8Repr::Vec4f(arr) => {
                    u32::from_be_bytes(arr.map(|f| (f * 255.0f32).floor() as u8))
                }
            }
            .into())
        }
    }
}
