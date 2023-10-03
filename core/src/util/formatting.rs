use std::collections::HashMap;

use anyhow::{anyhow, Ok};

#[derive(Clone)]
///`string_value = CODE_PREFIX + code`
pub struct Formatting<'a> {
    name: &'a str,
    code: char,
    modifier: bool,
    color_index: i32,
    color_value: Option<u32>,
}

impl Formatting<'_> {
    const CODE_PREFIX: char = '§';

    const BLACK: Self = Self {
        name: "BLACK",
        code: '0',
        modifier: false,
        color_index: 0,
        color_value: Some(0),
    };
    const DARK_BLUE: Self = Self {
        name: "DARK_BLUE",
        code: '1',
        modifier: false,
        color_index: 1,
        color_value: Some(170),
    };
    const DARK_GREEN: Self = Self {
        name: "DARK_GREEN",
        code: '2',
        modifier: false,
        color_index: 2,
        color_value: Some(43520),
    };
    const DARK_AQUA: Self = Self {
        name: "DARK_AQUA",
        code: '3',
        modifier: false,
        color_index: 3,
        color_value: Some(43690),
    };
    const DARK_RED: Self = Self {
        name: "DARK_RED",
        code: '4',
        modifier: false,
        color_index: 4,
        color_value: Some(11141120),
    };
    const DARK_PURPLE: Self = Self {
        name: "DARK_PURPLE",
        code: '5',
        modifier: false,
        color_index: 5,
        color_value: Some(11141290),
    };
    const GOLD: Self = Self {
        name: "GOLD",
        code: '6',
        modifier: false,
        color_index: 6,
        color_value: Some(16755200),
    };
    const GRAY: Self = Self {
        name: "GRAY",
        code: '7',
        modifier: false,
        color_index: 7,
        color_value: Some(11184810),
    };
    const DARK_GRAY: Self = Self {
        name: "DARK_GRAY",
        code: '8',
        modifier: false,
        color_index: 8,
        color_value: Some(5592405),
    };
    const BLUE: Self = Self {
        name: "BLUE",
        code: '9',
        modifier: false,
        color_index: 9,
        color_value: Some(5592575),
    };
    const GREEN: Self = Self {
        name: "GREEN",
        code: 'a',
        modifier: false,
        color_index: 10,
        color_value: Some(5635925),
    };
    const AQUA: Self = Self {
        name: "GREEN",
        code: 'b',
        modifier: false,
        color_index: 11,
        color_value: Some(5636095),
    };
    const RED: Self = Self {
        name: "RED",
        code: 'c',
        modifier: false,
        color_index: 12,
        color_value: Some(16733525),
    };
    const LIGHT_PURPLE: Self = Self {
        name: "LIGHT_PURPLE",
        code: 'd',
        modifier: false,
        color_index: 13,
        color_value: Some(16733695),
    };
    const YELLOW: Self = Self {
        name: "YELLOW",
        code: 'e',
        modifier: false,
        color_index: 14,
        color_value: Some(16777045),
    };
    const WHITE: Self = Self {
        name: "WHITE",
        code: 'f',
        modifier: false,
        color_index: 15,
        color_value: Some(16777215),
    };
    const OBFUSCATED: Self = Self {
        name: "OBFUSCATED",
        code: 'k',
        modifier: true,
        color_index: -1,
        color_value: None,
    };
    const BOLD: Self = Self {
        name: "BOLD",
        code: 'l',
        modifier: true,
        color_index: -1,
        color_value: None,
    };
    const STRIKETHROUGH: Self = Self {
        name: "STRIKETHROUGH",
        code: 'm',
        modifier: true,
        color_index: -1,
        color_value: None,
    };
    const UNDERLINE: Self = Self {
        name: "UNDERLINE",
        code: 'n',
        modifier: true,
        color_index: -1,
        color_value: None,
    };
    const ITALIC: Self = Self {
        name: "ITALIC",
        code: 'o',
        modifier: true,
        color_index: -1,
        color_value: None,
    };
    const RESET: Self = Self {
        name: "RESET",
        code: 'r',
        modifier: false,
        color_index: -1,
        color_value: None,
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

    pub fn try_from_name(name: &str) -> anyhow::Result<Self> {
        let map: HashMap<&str, Self> = Self::LIST.into_iter().map(|fmt| (fmt.name, fmt)).collect();
        let ret: anyhow::Result<Self> = match map.get(name) {
            Some(x) => Ok(x.clone()),
            None => Err(anyhow!("Formatting key {} not found!", name)),
        };
        ret
    }

    pub fn try_from_color_index(index: i32) -> anyhow::Result<Self> {
        if index < 0 {
            Ok(Self::RESET)
        } else {
            let map: HashMap<i32, Self> = Self::LIST
                .into_iter()
                .map(|fmt| (fmt.color_index, fmt))
                .collect();
            let ret: anyhow::Result<Self> = match map.get(&index) {
                Some(x) => Ok(x.clone()),
                None => Err(anyhow!("Color index {} not found!", index)),
            };
            ret
        }
    }

    pub fn try_from_code(code: char) -> anyhow::Result<Self> {
        let c: char = code.to_ascii_lowercase();
        let map: HashMap<char, Self> = Self::LIST.into_iter().map(|fmt| (fmt.code, fmt)).collect();
        let ret: anyhow::Result<Self> = match map.get(&c) {
            Some(x) => Ok(x.clone()),
            None => Err(anyhow!("Code {} not found!", c)),
        };
        ret
    }

    pub fn color_value(&self) -> Option<u32> {
        self.color_value
    }

    pub fn name(&self) -> &str {
        self.name
    }
}

impl super::StringIdentifiable for Formatting<'_> {
    fn as_string(&self) -> String {
        let mut ret: String = String::from(Self::CODE_PREFIX);
        ret.push(self.code);
        ret
    }
}

///Pre-installed formattings.
pub enum FormattingEnum {
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

impl FormattingEnum {
    ///Pure shit.
    pub fn to_formatting(e: Self) -> Formatting<'static> {
        let f: Formatting = match e {
            Self::Black => Formatting::BLACK,
            Self::DarkBlue => Formatting::DARK_BLUE,
            Self::DarkGreen => Formatting::DARK_GREEN,
            Self::DarkAqua => Formatting::DARK_AQUA,
            Self::DarkRed => Formatting::DARK_RED,
            Self::DarkPurple => Formatting::DARK_PURPLE,
            Self::Gold => Formatting::GOLD,
            Self::Gray => Formatting::GRAY,
            Self::DarkGray => Formatting::DARK_GRAY,
            Self::Blue => Formatting::BLUE,
            Self::Green => Formatting::GREEN,
            Self::Aqua => Formatting::AQUA,
            Self::Red => Formatting::RED,
            Self::LightPurple => Formatting::LIGHT_PURPLE,
            Self::Yellow => Formatting::YELLOW,
            Self::White => Formatting::WHITE,
            Self::Obfuscated => Formatting::OBFUSCATED,
            Self::Bold => Formatting::BOLD,
            Self::Strikethrough => Formatting::STRIKETHROUGH,
            Self::Underline => Formatting::UNDERLINE,
            Self::Italic => Formatting::ITALIC,
            Self::Reset => Formatting::RESET,
        };
        f
    }
}
