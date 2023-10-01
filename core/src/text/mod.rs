use anyhow::{anyhow, Ok};

///TODO: Implement net.minecraft.text.Text
pub trait Text {
    fn style(&self) -> &Style;
}

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
    ///TODO: Implement net.minecraft.util.Identifier
    font: Option<()>,
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
            Ok(Self { rgb: i, name: None })
        } else {
            Err(anyhow!("121345"))
        }
    }
}

pub enum Formatting {
    
}