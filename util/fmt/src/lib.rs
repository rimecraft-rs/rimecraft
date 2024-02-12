//! Minecraft `Formatting` in Rust.

use std::{fmt::Display, ops::Deref, sync::OnceLock};

use regex_lite::Regex;
use rgb::RGB8;

/// Color index of a formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorIndex(pub Option<u32>);

impl From<ColorIndex> for i32 {
    #[inline]
    fn from(ColorIndex(value): ColorIndex) -> Self {
        value.map_or(-1, |val| val as i32)
    }
}

impl TryFrom<i32> for ColorIndex {
    type Error = Error;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(Self(None)),
            0.. => Ok(Self(Some(value as u32))),
            _ => Err(Error::InvalidColorIndex(value)),
        }
    }
}

static SANITIZE_REGEX: OnceLock<Regex> = OnceLock::new();

macro_rules! formattings {
    ($($i:ident => $n:literal, $ln:literal, $sn:literal, $c:literal, $m:expr, $ci:literal, $cv:expr),*$(,)?) => {
        /// An enum holding formattings.
        ///
        /// There are two types of formattings, color and modifier.
        /// Color formattings are associated with a specific color,
        /// while modifier formattings modify the style, such as by
        /// bolding the text.
        ///
        /// [`Self::Reset`] is a special formatting and is not
        /// classified as either of these two.
        #[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        #[doc(alias = "ChatFormatting")]
        pub enum Formatting {
            $(#[doc = "The formatting."] $i),*
        }

        impl Formatting {
            /// The raw uppercase name of the formatting.
            ///
            /// # Examples
            ///
            /// ```
            /// # use rimecraft_fmt::Formatting;
            /// assert_eq!(Formatting::DarkBlue.raw_name(), "DARK_BLUE");
            /// ```
            #[inline]
            pub const fn raw_name(self) -> &'static str {
                match self {
                    $(Formatting::$i => $n),*
                }
            }

            /// Returns the code to be placed after the [`Self::CODE_PREFIX`]
            /// when this format is converted to a string.
            ///
            /// # Examples
            ///
            /// ```
            /// # use rimecraft_fmt::Formatting;
            /// assert_eq!(Formatting::DarkBlue.code(), '1');
            /// ```
            #[inline]
            pub const fn code(self) -> char {
                match self {
                    $(Formatting::$i => $c),*
                }
            }

            /// Returns the color index for the formatting.
            ///
            /// # Examples
            ///
            /// ```
            /// # use rimecraft_fmt::{ColorIndex, Formatting};
            /// assert_eq!(Formatting::DarkBlue.color_index(), ColorIndex(Some(1)));
            /// ```
            pub const fn color_index(self) -> ColorIndex {
                let value = match self { $(Formatting::$i => $ci),* };
                match value {
                    -1 => ColorIndex(None),
                    0.. => ColorIndex(Some(value as u32)),
                    _ => unreachable!(),
                }
            }

            /// Returns `true` if the formatting is a modifier.
            #[inline]
            pub const fn is_modifier(self) -> bool {
                match self {
                    $(Formatting::$i => $m),*
                }
            }

            /// Returns the color of the formatted text, or
            /// `None` if the formatting has no associated color.
            #[inline]
            pub const fn color_value(self) -> Option<RGB8> {
                if let Some(value) = match self { $(Formatting::$i => $cv),* }
                {
                    let value: u32 = value;
                    let [_, r, g, b] = value.to_be_bytes();
                    Some(RGB8 { r, g, b })
                } else {
                    None
                }
            }

            /// Returns the name of the formatting.
            ///
            /// # Examples
            ///
            /// ```
            /// # use rimecraft_fmt::Formatting;
            /// assert_eq!(Formatting::DarkBlue.name(), "dark_blue");
            /// ```
            #[inline]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Formatting::$i => $ln),*
                }
            }

            const VALUES: &'static [Self] = &[$(Self::$i),*];
        }

        impl TryFrom<ColorIndex> for Formatting {
            type Error = Error;

            fn try_from(ColorIndex(value): ColorIndex) -> Result<Self, Self::Error> {
                let Some(value) = value else { return Ok(Self::Reset) };
                let value = value as i32;
                $(if value == $ci {
                    return Ok(Self::$i);
                })*
                Err(Error::InvalidColorIndex(value))
            }
        }

        impl std::str::FromStr for Formatting {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if let Some(code) = s.strip_prefix(Self::CODE_PREFIX) {
                    return code.chars().next().ok_or(Error::InvalidCode(Self::CODE_PREFIX)).and_then(|c| c.try_into());
                }
                let s = s.to_ascii_lowercase();
                let s = SANITIZE_REGEX.get_or_init(|| Regex::new("[^a-z]").unwrap()).replace_all(&s, "");
                match s.as_ref() {
                    $($sn => Ok(Self::$i),)*
                    _ => Err(Error::InvalidName(s.into_owned())),
                }
            }
        }

        impl TryFrom<char> for Formatting {
            type Error = Error;

            fn try_from(c: char) -> Result<Self, Self::Error> {
                match c {
                    $($c => Ok(Self::$i),)*
                    _ => Err(Error::InvalidCode(c)),
                }
            }
        }
    };
}

formattings! {
    // Colors
    Black => "BLACK", "black", "black", '0', false, 0, Some(0x0),
    DarkBlue => "DARK_BLUE", "dark_blue", "darkblue", '1', false, 1, Some(0xaa),
    DarkGreen => "DARK_GREEN", "dark_green", "darkgreen", '2', false, 2, Some(0xaa00),
    DarkAqua => "DARK_AQUA", "dark_aqua", "darkaqua", '3', false, 3, Some(0xaaaa),
    DarkRed => "DARK_RED", "dark_red", "darkred", '4', false, 4, Some(0xaa0000),
    DarkPurple => "DARK_PURPLE", "dark_purple", "darkpurple", '5', false, 5, Some(0xaa00aa),
    Gold => "GOLD", "gold", "gold", '6', false, 6, Some(0xffaa00),
    Gray => "GRAY", "gray", "gray", '7', false, 7, Some(0xaaaaaa),
    DarkGray => "DARK_GRAY", "dark_gray", "darkgray", '8', false, 8, Some(0x555555),
    Blue => "BLUE", "blue", "blue", '9', false, 9, Some(0x5555ff),
    Green => "GREEN", "green", "green", 'a', false, 10, Some(0x55ff55),
    Aqua => "AQUA", "aqua", "aqua", 'b', false, 11, Some(0x55ffff),
    Red => "RED", "red", "red", 'c', false, 12, Some(0xff5555),
    LightPurple => "LIGHT_PURPLE", "light_purple", "lightpurple", 'd', false, 13, Some(0xff55ff),
    Yellow => "YELLOW", "yellow", "yellow", 'e', false, 14, Some(0xffff55),
    White => "WHITE", "white", "white", 'f', false, 15, Some(0xffffff),

    // Modifiers
    Obfuscated => "OBFUSCATED", "obfuscated", "obfuscated", 'k', true, -1, None,
    Bold => "BOLD", "bold", "bold", 'l', true, -1, None,
    Strikethrough => "STRIKETHROUGH", "strikethrough", "strikethrough", 'm', true, -1, None,
    Underline => "UNDERLINE", "underline", "underline", 'n', true, -1, None,
    Italic => "ITALIC", "italic", "italic", 'o', true, -1, None,

    // Special
    Reset => "RESET", "reset", "reset", 'r', false, -1, None,
}

/// An error returned when parsing a formatting.
#[derive(Debug)]
#[allow(variant_size_differences)]
pub enum Error {
    /// No matching color index found.
    InvalidColorIndex(i32),
    /// Invalid code.
    InvalidCode(char),
    /// Invalid name.
    InvalidName(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidColorIndex(i) => write!(f, "no matching color index found: {}", i),
            Error::InvalidCode(c) => write!(f, "invalid code: {}", c),
            Error::InvalidName(n) => write!(f, "invalid name: {}", n),
        }
    }
}

impl std::error::Error for Error {}

impl Formatting {
    /// The prefix of formatting codes.
    pub const CODE_PREFIX: char = '§';

    /// Whether the formatting is associated with a color.
    #[inline]
    pub const fn is_color(self) -> bool {
        !self.is_modifier() && !matches!(self, Self::Reset)
    }

    /// Get an iterator iterates over names of all formattings.
    #[inline]
    pub fn names() -> Names {
        Names {
            inner: Self::VALUES.iter(),
        }
    }
}

impl AsRef<str> for Formatting {
    #[inline]
    fn as_ref(&self) -> &str {
        self.name()
    }
}

/// The iterator returned by [`Formatting::names`].
#[derive(Debug)]
pub struct Names {
    inner: std::slice::Iter<'static, Formatting>,
}

impl Iterator for Names {
    type Item = Name;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|val| Name { value: *val })
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

impl Display for Formatting {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", Self::CODE_PREFIX, self.code())
    }
}

#[cfg(test)]
mod tests;
