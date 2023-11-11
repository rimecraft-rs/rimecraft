use std::{collections::HashMap, fmt::Display, ops::Deref, str::FromStr};

use fastnbt::de;
use once_cell::sync::Lazy;

use super::RGB;

/// An index of a color in [`Formatting`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ColorIndex {
    value: i8,
}

impl ColorIndex {
    /// Returns the color index for the given value.
    #[inline]
    pub fn new(value: Option<u8>) -> Self {
        Self {
            value: value.map(|e| e as i8).unwrap_or(-1),
        }
    }

    /// Returns the value of the color index if valid.
    #[inline]
    pub fn value(self) -> Option<u8> {
        if self.value == -1 {
            None
        } else {
            Some(self.value as u8)
        }
    }
}

impl Default for ColorIndex {
    #[inline]
    fn default() -> Self {
        Self { value: -1 }
    }
}

macro_rules! formattings {
    ($( $i:ident => $n:literal, $c:literal, $m:expr, $ci:literal, $cv:expr ),+,) => {
        /// An enum holding formattings.
        ///
        /// There are two types of formattings, color and modifier. Color formattings
        /// are associated with a specific color, while modifier formattings modify the
        /// style, such as by bolding the text. [`Self::Reset`] is a special formatting
        /// and is not classified as either of these two.
        #[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        #[repr(u8)]
        pub enum Formatting {
            $(
                $i,
            )*
        }

        impl Formatting {
            #[inline]
            fn name_raw(self) -> &'static str {
                match self {
                    $(
                        Formatting::$i => $n,
                    )*
                }
            }

            /// Returns the code to be placed after the [`Self::CODE_PREFIX`]
            /// when this format is converted to a string.
            #[inline]
            pub fn code(self) -> char {
                match self {
                    $(
                        Formatting::$i => $c,
                    )*
                }
            }

            /// Returns the color index for the formatting.06
            #[inline]
            pub fn color_index(self) -> ColorIndex {
                match self {
                    $(
                        Formatting::$i => ColorIndex { value: $ci },
                    )*
                }
            }

            /// Returns `true` if the formatting is a modifier.
            #[inline]
            pub fn is_modifier(self) -> bool {
                match self {
                    $(
                        Formatting::$i => $m,
                    )*
                }
            }

            /// Returns the color of the formatted text, or
            /// `None` if the formatting has no associated color.
            #[inline]
            pub fn color_value(self) -> Option<RGB> {
                match self {
                    $(
                        Formatting::$i => $cv.map(RGB::new),
                    )*
                }
            }

            /// Returns all values of this enum.
            fn values() -> &'static [Self] {
                static VALS: Lazy<Vec<Formatting>> = Lazy::new(|| {
                    vec![
                        $(
                            Formatting::$i,
                        )*
                    ]
                });
                &VALS
            }
        }
    };
}

formattings! {
    // Colors
    Black => "BLACK", '0', false, 0, Some(0x0),
    DarkBlue => "DARK_BLUE", '1', false, 1, Some(0xaa),
    DarkGreen => "DARK_GREEN", '2', false, 2, Some(0xaa00),
    DarkAqua => "DARK_AQUA", '3', false, 3, Some(0xaaaa),
    DarkRed => "DARK_RED", '4', false, 4, Some(0xaa0000),
    DarkPurple => "DARK_PURPLE", '5', false, 5, Some(0xaa00aa),
    Gold => "GOLD", '6', false, 6, Some(0xffaa00),
    Gray => "GRAY", '7', false, 7, Some(0xaaaaaa),
    DarkGray => "DARK_GRAY", '8', false, 8, Some(0x555555),
    Blue => "BLUE", '9', false, 9, Some(0x5555ff),
    Green => "GREEN", 'a', false, 10, Some(0x55ff55),
    Aqua => "AQUA", 'b', false, 11, Some(0x55ffff),
    Red => "RED", 'c', false, 12, Some(0xff5555),
    LightPurple => "LIGHT_PURPLE", 'd', false, 13, Some(0xff55ff),
    Yellow => "YELLOW", 'e', false, 14, Some(0xffff55),
    White => "WHITE", 'f', false, 15, Some(0xffffff),

    // Modifiers
    Obfuscated => "OBFUSCATED", 'k', true, -1, None,
    Bold => "BOLD", 'l', true, -1, None,
    Strikethrough => "STRIKETHROUGH", 'm', true, -1, None,
    Underline => "UNDERLINE", 'n', true, -1, None,
    Italic => "ITALIC", 'o', true, -1, None,

    // Special
    Reset => "RESET", 'r', false, -1, None,
}

/// An error returned when parsing a formatting.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// No matching color index found.
    #[error("no matching color index {0:?} found")]
    NoMatchingColorIndex(ColorIndex),
    /// Invalid code.
    #[error("invalid code: {0}")]
    InvalidCode(char),
    /// Invalid name.
    #[error("invalid name: {0}")]
    InvalidName(String),
}

impl Formatting {
    /// The prefix of formatting codes.
    pub const CODE_PREFIX: char = '§';

    /// Returns `true` if the formatting is associated with
    /// a color, `false` otherwise.
    #[inline]
    pub fn is_color(self) -> bool {
        !self.is_modifier() && self != Self::Reset
    }

    /// Name of the formatting.
    #[inline]
    pub fn name(self) -> &'static str {
        static NAMING_MAP: Lazy<Vec<String>> = Lazy::new(|| {
            Formatting::values()
                .iter()
                .map(|e| e.name_raw().to_ascii_lowercase())
                .collect()
        });
        NAMING_MAP.get(self as u8 as usize).unwrap()
    }

    /// Get an iterator iterates over names of all formattings.
    #[inline]
    pub fn names() -> Names {
        Names {
            iter: Self::values().iter(),
        }
    }
}

/// Sanitize a formatting name.
#[inline]
fn name_sanitize(name: &str) -> String {
    lazy_regex::regex_replace_all!("[^a-z]", &name.to_lowercase(), "").into_owned()
}

impl AsRef<str> for Formatting {
    #[inline]
    fn as_ref(&self) -> &str {
        self.name()
    }
}

impl FromStr for Formatting {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static NAME_MAP: Lazy<HashMap<String, Formatting>> = Lazy::new(|| {
            Formatting::values()
                .iter()
                .map(|e| (name_sanitize(e.name_raw()), *e))
                .collect()
        });
        NAME_MAP
            .get(&name_sanitize(s))
            .copied()
            .ok_or_else(|| Error::InvalidName(s.to_owned()))
    }
}

impl Display for Formatting {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        f.write_char(Self::CODE_PREFIX)?;
        f.write_char(self.code())
    }
}

impl TryFrom<ColorIndex> for Formatting {
    type Error = Error;

    #[inline]
    fn try_from(value: ColorIndex) -> Result<Self, Self::Error> {
        if value.value().is_none() {
            Ok(Self::Reset)
        } else {
            static CI_MAP: Lazy<HashMap<ColorIndex, Formatting>> = Lazy::new(|| {
                Formatting::values()
                    .iter()
                    .map(|e| (e.color_index(), *e))
                    .collect()
            });
            CI_MAP
                .get(&value)
                .copied()
                .ok_or(Error::NoMatchingColorIndex(value))
        }
    }
}

impl TryFrom<char> for Formatting {
    type Error = Error;

    #[inline]
    fn try_from(value: char) -> Result<Self, Self::Error> {
        static CHAR_MAP: Lazy<HashMap<char, Formatting>> = Lazy::new(|| {
            Formatting::values()
                .iter()
                .map(|e| (e.code(), *e))
                .collect()
        });
        CHAR_MAP
            .get(&value.to_ascii_lowercase())
            .copied()
            .ok_or(Error::InvalidCode(value))
    }
}

/// The iterator returned by [`Formatting::names`].
#[derive(Debug)]
pub struct Names {
    iter: std::slice::Iter<'static, Formatting>,
}

impl Iterator for Names {
    type Item = Name;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|val| Name { value: *val })
    }
}

/// Item of [`Names`].
#[derive(Debug)]
pub struct Name {
    value: Formatting,
}

impl Name {
    /// Returns whether the targeting formatting is a color.
    #[inline]
    pub fn is_color(&self) -> bool {
        self.value.is_color()
    }

    /// Returns whether the targeting formatting is a modifier.
    #[inline]
    pub fn is_modifier(&self) -> bool {
        self.value.is_modifier()
    }
}

impl Deref for Name {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value.name()
    }
}

impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}
