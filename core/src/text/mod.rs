use std::{
    any::TypeId,
    borrow::Cow,
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use rimecraft_event::Event;
use rimecraft_primitives::ErasedSerDeUpdate;

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
    // TODO: Implement net.minecraft.text.HoverEvent
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

/// Represents an RGB color of a [`Text`].
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.TextColor` (yarn).
#[derive(Debug, Hash)]
pub struct Color {
    /// 24-bit color.
    rgb: u32,
    name: Option<String>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    pub fn try_parse(name: String) -> anyhow::Result<Self> {
        if let Some(value) = name.strip_prefix(Self::RGB_PREFIX) {
            Ok(Self::from_rgb(value.parse()?))
        } else {
            let f = crate::util::formatting::Formatting::try_from_name(&name)?;
            Ok(Self {
                rgb: f
                    .color_value()
                    .ok_or_else(|| anyhow::anyhow!("no valid color value"))?,
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

trait UpdDebug: ErasedSerDeUpdate + Debug {}
impl<T> UpdDebug for T where T: ?Sized + ErasedSerDeUpdate + Debug {}

pub struct HoverEvent {
    contents: (Box<dyn UpdDebug + Send + Sync>, TypeId),
    action: &'static HoverEventAction,
    hash: u64,
}

impl HoverEvent {
    pub fn new<T>(action: &'static HoverEventAction, contents: T) -> Self
    where
        T: ErasedSerDeUpdate + Debug + Hash + Send + Sync + 'static,
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
            Some(unsafe { &*(&*self.contents.0 as *const (dyn UpdDebug + Send + Sync) as *const T) })
        } else {
            None
        }
    }

    #[inline]
    pub fn value_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if TypeId::of::<T>() == self.contents.1 {
            Some(unsafe {
                &mut *(&mut *self.contents.0 as *mut (dyn UpdDebug + Send + Sync) as *mut T)
            })
        } else {
            None
        }
    }
}

impl Serialize for HoverEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl Hash for HoverEvent {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
        self.action.hash(state);
    }
}

impl Debug for HoverEvent {
    #[inline]
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
    name: Cow<'static, str>,
    parsable: bool,
}

/// An event to process actions on initialize.
static HE_ACTIONS: RwLock<Event<dyn Fn(&mut Vec<HoverEventAction>)>> =
    RwLock::new(Event::new(|listeners| {
        Box::new(move |actions| {
            for listener in listeners {
                listener(actions)
            }
        })
    }));

static HE_MAPPING: Lazy<HashMap<String, HoverEventAction>> = Lazy::new(|| {
    HE_ACTIONS.write().register(Box::new(|v| {
        let mut vec = vec![
            HoverEventAction::SHOW_TEXT,
            HoverEventAction::SHOW_ITEM,
            HoverEventAction::SHOW_ENTITY,
        ];
        v.append(&mut vec);
    }));

    let mut vec = Vec::new();
    HE_ACTIONS.read().invoker()(&mut vec);
    vec.into_iter().map(|v| (v.name().to_owned(), v)).collect()
});

impl HoverEventAction {
    pub const SHOW_TEXT: Self = Self {
        name: Cow::Borrowed("show_text"),
        parsable: true,
    };

    pub const SHOW_ITEM: Self = Self {
        name: Cow::Borrowed("show_item"),
        parsable: true,
    };

    pub const SHOW_ENTITY: Self = Self {
        name: Cow::Borrowed("show_entity"),
        parsable: true,
    };

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn is_parsable(&self) -> bool {
        self.parsable
    }

    #[inline]
    pub fn from_name(name: &str) -> Option<&'static Self> {
        HE_MAPPING.get(name)
    }
}

impl Debug for HoverEventAction {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<action {}>", self.name)
    }
}

impl Serialize for HoverEventAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.name().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for &'static HoverEventAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static VARIANTS: Lazy<Vec<&'static str>> =
            Lazy::new(|| HE_MAPPING.iter().map(|v| v.0.as_str()).collect());
        let value = String::deserialize(deserializer)?;

        use serde::de::Error;

        HoverEventAction::from_name(&value)
            .ok_or_else(|| D::Error::unknown_variant(&value, &VARIANTS))
    }
}
