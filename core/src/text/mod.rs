use std::{
    any::TypeId,
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use once_cell::sync::Lazy;

use anyhow::anyhow;

///TODO: Implement net.minecraft.text.Text
pub trait Text {
    fn style(&self) -> &Style;
}

///The style of a [`Text`].\
///A style is immutable.
pub struct Style {
    color: Option<Color>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    click: Option<ClickEvent>,
    ///TODO: Implement net.minecraft.text.HoverEvent
    hover: Option<()>,
    insertion: Option<String>,
    font: Option<rimecraft_primitives::Id>,
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

#[derive(Debug, Hash)]
pub struct Color {
    ///24-bit color.
    rgb: u32,
    name: Option<String>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    pub fn try_parse(name: String) -> anyhow::Result<Self> {
        if name.starts_with(Self::RGB_PREFIX) {
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

    #[inline]
    pub fn from_rgb(rgb: u32) -> Self {
        Self { rgb, name: None }
    }

    #[inline]
    pub fn new(rgb: u32, name: String) -> Self {
        Self {
            rgb,
            name: Some(name),
        }
    }

    #[inline]
    fn to_hex_str(&self) -> String {
        return format!("{}{:06X}", Self::RGB_PREFIX, self.rgb);
    }

    #[inline]
    pub fn name(&self) -> String {
        match &(self.name) {
            Some(x) => x.clone(),
            None => self.to_hex_str(),
        }
    }

    pub fn try_from_formatting(formatting: &super::formatting::Formatting) -> anyhow::Result<Self> {
        if formatting.is_color() {
            Ok(Self {
                rgb: formatting.color_value().unwrap(),
                name: Some(String::from(formatting.name().clone())),
            })
        } else {
            Err(anyhow!("Not a valid color!"))
        }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.rgb == other.rgb
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        write!(f, "{name}")
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct ClickEvent {
    action: ClickEventAction,
    value: String,
}

impl ClickEvent {
    #[inline]
    pub fn action(&self) -> ClickEventAction {
        self.action
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[inline]
    pub fn new(action: ClickEventAction, value: String) -> Self {
        Self { action, value }
    }
}

impl Debug for ClickEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action = self.action().name();
        let value = self.value();

        write!(f, "ClickEvent{{action={action}, value='{value}'}}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClickEventAction {
    OpenUrl,
    OpenFile,
    RunCommand,
    SuggestCommand,
    ChangePage,
    CopyToClipboard,
}

impl ClickEventAction {
    const VALUES: [Self; 6] = [
        Self::OpenUrl,
        Self::OpenFile,
        Self::RunCommand,
        Self::SuggestCommand,
        Self::ChangePage,
        Self::CopyToClipboard,
    ];

    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::OpenUrl => "open_url",
            Self::OpenFile => "open_file",
            Self::RunCommand => "run_command",
            Self::SuggestCommand => "suggest_command",
            Self::ChangePage => "change_page",
            Self::CopyToClipboard => "copy_to_clipboard",
        }
    }

    #[inline]
    pub fn user_definable(self) -> bool {
        !matches!(self, Self::OpenFile)
    }

    #[inline]
    pub fn from_name(name: &str) -> Option<Self> {
        static MAP: Lazy<HashMap<String, ClickEventAction>> = Lazy::new(|| {
            ClickEventAction::VALUES
                .into_iter()
                .map(|value| (value.name().to_owned(), value))
                .collect()
        });

        MAP.get(name).copied()
    }
}

pub struct HoverEvent {
    contents: (Box<dyn Debug + Send + Sync>, TypeId),
    action: HoverEventAction,
    hash: u64,
}

impl HoverEvent {
    pub fn new<T>(action: HoverEventAction, contents: T) -> Self
    where
        T: Debug + Hash + Send + Sync + 'static,
    {
        let mut hasher = DefaultHasher::new();
        contents.hash(&mut hasher);

        Self {
            contents: (Box::new(contents), TypeId::of::<T>()),
            action,
            hash: hasher.finish(),
        }
    }

    #[inline]
    pub fn action(&self) -> &HoverEventAction {
        &self.action
    }

    #[inline]
    pub fn value<T: 'static>(&self) -> Option<&T> {
        if TypeId::of::<T>() == self.contents.1 {
            Some(unsafe { &*(&*self.contents.0 as *const (dyn Debug + Send + Sync) as *const T) })
        } else {
            None
        }
    }

    #[inline]
    pub fn value_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if TypeId::of::<T>() == self.contents.1 {
            Some(unsafe {
                &mut *(&mut *self.contents.0 as *mut (dyn Debug + Send + Sync) as *mut T)
            })
        } else {
            None
        }
    }
}

impl Hash for HoverEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
        self.action.hash(state);
    }
}

impl Debug for HoverEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HoverEvent{{action={:?},value={:?}}}",
            self.action, self.contents
        )
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct HoverEventAction {
    name: String,
    parsable: bool,
}

impl HoverEventAction {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn is_parsable(&self) -> bool {
        self.parsable
    }
}

impl Debug for HoverEventAction {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<action {} >", self.name)
    }
}
