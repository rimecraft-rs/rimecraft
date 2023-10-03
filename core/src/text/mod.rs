use anyhow::{anyhow, Ok};

use crate::util;

///TODO: Implement net.minecraft.text.Text
pub trait Text {
    fn style(&self) -> &Style;
}

///The style of a [`Text`].\
///A style is immutable.
pub struct Style {
    ///TODO: Implement net.minecraft.text.TextColor
    color: Option<()>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    ///TODO: Implement net.minecraft.text.ClickEvent
    click: Option<()>,
    ///TODO: Implement net.minecraft.text.HoverEvent
    hover: Option<()>,
    insertion: Option<String>,
    font: Option<util::Id>,
}

impl Style {
    const EMPTY: Style = Style {
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
}

pub struct Color {
    ///24-bit color.
    rgb: u32,
    name: Option<String>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    pub fn try_parse(name: String) -> anyhow::Result<Self> {
        if (name.starts_with(Self::RGB_PREFIX)) {
            let i: u32 = str::parse(&name[1..])?;
            Ok(Self::from_rgb(i))
        } else {
            let f = crate::util::formatting::Formatting::try_from_name(&name)?;
            let cv: u32 = match f.color_value() {
                Some(x) => x,
                None => return Err(anyhow!("No valid color value!")),
            };
            Ok(Self {
                rgb: cv,
                name: Some(String::from(f.name())),
            })
        }
    }

    pub fn from_rgb(rgb: u32) -> Self {
        Self { rgb, name: None }
    }

    pub fn new(rgb: u32, name: String) -> Self {
        Self {
            rgb,
            name: Some(name),
        }
    }

    fn to_hex_str(&self) -> String {
        return format!("{}{:06X}", Self::RGB_PREFIX, self.rgb);
    }

    pub fn name(&self) -> String {
        match &(self.name) {
            Some(x) => x.clone(),
            None => self.to_hex_str(),
        }
    }
}
