use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// A type holding formattings.
///
/// There are two types of formattings, color and modifier. Color formattings
/// are associated with a specific color, while modifier formattings modify the
/// style, such as by bolding the text. [`Self::RESET`] is a special formatting
/// and is not classified as either of these two.
pub struct Formatting {
    name: Cow<'static, str>,
    code: char,
    modifier: bool,
    color_index: i32,
    color_value: Option<u32>,
    enum_v: Enum,
}

impl Formatting {
    const CODE_PREFIX: char = '§';

    pub const BLACK: Self = Self {
        name: Cow::Borrowed("BLACK"),
        code: '0',
        modifier: false,
        color_index: 0,
        color_value: Some(0x000000),
        enum_v: Enum::Black,
    };

    pub const DARK_BLUE: Self = Self {
        name: Cow::Borrowed("DARK_BLUE"),
        code: '1',
        modifier: false,
        color_index: 1,
        color_value: Some(0x0000AA),
        enum_v: Enum::DarkBlue,
    };

    pub const DARK_GREEN: Self = Self {
        name: Cow::Borrowed("DARK_GREEN"),
        code: '2',
        modifier: false,
        color_index: 2,
        color_value: Some(0x00AA00),
        enum_v: Enum::DarkGreen,
    };

    pub const DARK_AQUA: Self = Self {
        name: Cow::Borrowed("DARK_AQUA"),
        code: '3',
        modifier: false,
        color_index: 3,
        color_value: Some(0x00AAAA),
        enum_v: Enum::DarkAqua,
    };

    pub const DARK_RED: Self = Self {
        name: Cow::Borrowed("DARK_RED"),
        code: '4',
        modifier: false,
        color_index: 4,
        color_value: Some(0xAA0000),
        enum_v: Enum::DarkRed,
    };

    pub const DARK_PURPLE: Self = Self {
        name: Cow::Borrowed("DARK_PURPLE"),
        code: '5',
        modifier: false,
        color_index: 5,
        color_value: Some(0xAA00AA),
        enum_v: Enum::DarkPurple,
    };

    pub const GOLD: Self = Self {
        name: Cow::Borrowed("GOLD"),
        code: '6',
        modifier: false,
        color_index: 6,
        color_value: Some(0xFFAA00),
        enum_v: Enum::Gold,
    };

    pub const GRAY: Self = Self {
        name: Cow::Borrowed("GRAY"),
        code: '7',
        modifier: false,
        color_index: 7,
        color_value: Some(0xAAAAAA),
        enum_v: Enum::Gray,
    };

    pub const DARK_GRAY: Self = Self {
        name: Cow::Borrowed("DARK_GRAY"),
        code: '8',
        modifier: false,
        color_index: 8,
        color_value: Some(0x555555),
        enum_v: Enum::DarkGray,
    };

    pub const BLUE: Self = Self {
        name: Cow::Borrowed("BLUE"),
        code: '9',
        modifier: false,
        color_index: 9,
        color_value: Some(0x5555FF),
        enum_v: Enum::Blue,
    };

    pub const GREEN: Self = Self {
        name: Cow::Borrowed("GREEN"),
        code: 'a',
        modifier: false,
        color_index: 10,
        color_value: Some(0x55FF55),
        enum_v: Enum::Green,
    };

    pub const AQUA: Self = Self {
        name: Cow::Borrowed("GREEN"),
        code: 'b',
        modifier: false,
        color_index: 11,
        color_value: Some(0x55FFFF),
        enum_v: Enum::Aqua,
    };

    pub const RED: Self = Self {
        name: Cow::Borrowed("RED"),
        code: 'c',
        modifier: false,
        color_index: 12,
        color_value: Some(0xFF5555),
        enum_v: Enum::Red,
    };

    pub const LIGHT_PURPLE: Self = Self {
        name: Cow::Borrowed("LIGHT_PURPLE"),
        code: 'd',
        modifier: false,
        color_index: 13,
        color_value: Some(0xFF55FF),
        enum_v: Enum::LightPurple,
    };

    pub const YELLOW: Self = Self {
        name: Cow::Borrowed("YELLOW"),
        code: 'e',
        modifier: false,
        color_index: 14,
        color_value: Some(0xFFFF55),
        enum_v: Enum::Yellow,
    };

    pub const WHITE: Self = Self {
        name: Cow::Borrowed("WHITE"),
        code: 'f',
        modifier: false,
        color_index: 15,
        color_value: Some(0xFFFFFF),
        enum_v: Enum::White,
    };

    pub const OBFUSCATED: Self = Self {
        name: Cow::Borrowed("OBFUSCATED"),
        code: 'k',
        modifier: true,
        color_index: -1,
        color_value: None,
        enum_v: Enum::Obfuscated,
    };

    pub const BOLD: Self = Self {
        name: Cow::Borrowed("BOLD"),
        code: 'l',
        modifier: true,
        color_index: -1,
        color_value: None,
        enum_v: Enum::Bold,
    };

    pub const STRIKETHROUGH: Self = Self {
        name: Cow::Borrowed("STRIKETHROUGH"),
        code: 'm',
        modifier: true,
        color_index: -1,
        color_value: None,
        enum_v: Enum::Strikethrough,
    };

    pub const UNDERLINE: Self = Self {
        name: Cow::Borrowed("UNDERLINE"),
        code: 'n',
        modifier: true,
        color_index: -1,
        color_value: None,
        enum_v: Enum::Underline,
    };

    pub const ITALIC: Self = Self {
        name: Cow::Borrowed("ITALIC"),
        code: 'o',
        modifier: true,
        color_index: -1,
        color_value: None,
        enum_v: Enum::Italic,
    };

    pub const RESET: Self = Self {
        name: Cow::Borrowed("RESET"),
        code: 'r',
        modifier: false,
        color_index: -1,
        color_value: None,
        enum_v: Enum::Reset,
    };

    const LIST: [Self; 22] = [
        Self::BLACK,
        Self::DARK_BLUE,
        Self::DARK_GREEN,
        Self::DARK_AQUA,
        Self::DARK_RED,
        Self::DARK_PURPLE,
        Self::GOLD,
        Self::GRAY,
        Self::DARK_GRAY,
        Self::BLUE,
        Self::GREEN,
        Self::AQUA,
        Self::RED,
        Self::LIGHT_PURPLE,
        Self::YELLOW,
        Self::WHITE,
        Self::OBFUSCATED,
        Self::BOLD,
        Self::STRIKETHROUGH,
        Self::UNDERLINE,
        Self::ITALIC,
        Self::RESET,
    ];

    pub fn try_from_name(name: &str) -> Result<&'static Self, Error> {
        use once_cell::sync::Lazy;

        static MAPPING: Lazy<HashMap<String, &'static Formatting>> = Lazy::new(|| {
            Formatting::LIST
                .iter()
                .map(|fmt| (fmt.name.clone().into_owned(), fmt))
                .collect()
        });

        MAPPING
            .get(name)
            .copied()
            .ok_or_else(|| Error::KeyNotFound {
                key: name.to_owned(),
            })
    }

    pub fn try_from_color_index(index: i32) -> Result<&'static Self, Error> {
        if index < 0 {
            Ok(&Self::RESET)
        } else {
            Self::LIST
                .iter()
                .find(|value| value.color_index == index)
                .ok_or(Error::ColorIndexNotFound { index })
        }
    }

    pub fn try_from_code(code: char) -> Result<&'static Self, Error> {
        let c: char = code.to_ascii_lowercase();
        Self::LIST
            .iter()
            .find(|value| value.code == c)
            .ok_or(Error::CodeNotFound { code })
    }

    #[inline]
    pub fn color_value(&self) -> Option<u32> {
        self.color_value
    }

    #[inline]
    pub fn name(&self) -> String {
        self.name.to_ascii_lowercase()
    }

    #[inline]
    pub fn is_modifier(&self) -> bool {
        self.modifier
    }

    #[inline]
    pub fn is_color(&self) -> bool {
        self.color_index >= 0
    }
}

impl serde::Serialize for Formatting {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.enum_v.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for &'static Formatting {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Enum::deserialize(deserializer)?.into())
    }
}

impl Display for Formatting {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", Self::CODE_PREFIX, self.code)
    }
}

/// Error variants of formatting.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("formatting key {key} not found")]
    KeyNotFound { key: String },
    #[error("color index {index} not found")]
    ColorIndexNotFound { index: i32 },
    #[error("code {code} not found")]
    CodeNotFound { code: char },
}

/// Pre-installed formattings.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum Enum {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

impl From<Enum> for &'static Formatting {
    #[inline]
    fn from(value: Enum) -> &'static Formatting {
        match value {
            Enum::Black => &Formatting::BLACK,
            Enum::DarkBlue => &Formatting::DARK_BLUE,
            Enum::DarkGreen => &Formatting::DARK_GREEN,
            Enum::DarkAqua => &Formatting::DARK_AQUA,
            Enum::DarkRed => &Formatting::DARK_RED,
            Enum::DarkPurple => &Formatting::DARK_PURPLE,
            Enum::Gold => &Formatting::GOLD,
            Enum::Gray => &Formatting::GRAY,
            Enum::DarkGray => &Formatting::DARK_GRAY,
            Enum::Blue => &Formatting::BLUE,
            Enum::Green => &Formatting::GREEN,
            Enum::Aqua => &Formatting::AQUA,
            Enum::Red => &Formatting::RED,
            Enum::LightPurple => &Formatting::LIGHT_PURPLE,
            Enum::Yellow => &Formatting::YELLOW,
            Enum::White => &Formatting::WHITE,
            Enum::Obfuscated => &Formatting::OBFUSCATED,
            Enum::Bold => &Formatting::BOLD,
            Enum::Strikethrough => &Formatting::STRIKETHROUGH,
            Enum::Underline => &Formatting::UNDERLINE,
            Enum::Italic => &Formatting::ITALIC,
            Enum::Reset => &Formatting::RESET,
        }
    }
}

impl From<&'static Formatting> for Enum {
    #[inline]
    fn from(value: &'static Formatting) -> Self {
        value.enum_v
    }
}
