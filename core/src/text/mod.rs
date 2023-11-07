use std::{
    any::TypeId,
    borrow::Cow,
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    str::FromStr,
    sync::Arc,
};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use rimecraft_event::Event;
use rimecraft_primitives::{id, ErasedSerDeUpdate, Id};

use super::formatting::Formatting;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not a valid color")]
    InvalidColor,
    #[error("no valid color value found")]
    ColorValueNotFound,
    #[error("unable to parse integer: {0}")]
    ParseInt(std::num::ParseIntError),
    #[error("formatting error: {0}")]
    Formatting(super::formatting::Error),
}

/// TODO: Implement net.minecraft.text.Text
pub trait Text {
    fn style(&self) -> &Style;
    fn siblings(&self) -> Vec<Box<dyn Text>>;
    /// TODO: Implement net.minecraft.text.OrderedText
    fn as_ordered_text(&self) -> ();
}

/// The style of a [`Text`].\
/// A style is immutable.
#[derive(PartialEq, Eq)]
pub struct Style {
    color: Option<Color>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    click: Option<ClickEvent>,
    // TODO: Implement net.minecraft.text.HoverEvent
    hover: Option<HoverEvent>,
    insertion: Option<String>,
    font: Option<rimecraft_primitives::Id>,
}

impl Style {
    const EMPTY: Self = Self {
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

    const DEFAULT_FONT_ID: &str = "default";

    #[inline]
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    #[inline]
    pub fn bold(&self) -> bool {
        self.bold.unwrap_or(false)
    }

    #[inline]
    pub fn italic(&self) -> bool {
        self.italic.unwrap_or(false)
    }

    #[inline]
    pub fn strikethrough(&self) -> bool {
        self.strikethrough.unwrap_or(false)
    }

    #[inline]
    pub fn underlined(&self) -> bool {
        self.underlined.unwrap_or(false)
    }

    #[inline]
    pub fn obfuscated(&self) -> bool {
        self.obfuscated.unwrap_or(false)
    }

    pub fn empty(&self) -> bool {
        self == &Self::EMPTY
    }

    pub fn click(&self) -> Option<&ClickEvent> {
        self.click.as_ref()
    }

    pub fn hover(&self) -> Option<&HoverEvent> {
        self.hover.as_ref()
    }

    pub fn insertion(&self) -> Option<&String> {
        self.insertion.as_ref()
    }

    pub fn font(&self) -> Cow<'_, Id> {
        self.font
            .as_ref()
            .map_or_else(|| Cow::Owned(id!(Self::DEFAULT_FONT_ID)), Cow::Borrowed)
    }
}

/// Represents an RGB color of a [`Text`].
///
/// This is immutable as a part of [`Style`].
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.TextColor` (yarn).
#[derive(Debug, Hash)]
pub struct Color {
    /// A 24-bit color.
    rgb: u32,
    name: Option<Cow<'static, str>>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    #[inline]
    pub fn from_rgb(rgb: u32) -> Self {
        Self { rgb, name: None }
    }

    #[inline]
    pub fn new(rgb: u32, name: Cow<'static, str>) -> Self {
        Self {
            rgb,
            name: Some(name),
        }
    }

    #[inline]
    fn as_hex_str(&self) -> String {
        format!("{}{:06X}", Self::RGB_PREFIX, self.rgb)
    }

    #[inline]
    pub fn name(&self) -> Cow<'_, str> {
        self.name
            .clone()
            .unwrap_or_else(|| Cow::Owned(self.as_hex_str()))
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(value) = s.strip_prefix(Self::RGB_PREFIX) {
            Ok(Self::from_rgb(value.parse().map_err(Error::ParseInt)?))
        } else {
            let f =
                crate::util::formatting::Formatting::try_from_name(s).map_err(Error::Formatting)?;
            Ok(Self {
                rgb: f.color_value().ok_or(Error::ColorValueNotFound)?,
                name: Some(Cow::Borrowed(f.name())),
            })
        }
    }
}

impl TryFrom<&'static Formatting> for Color {
    type Error = Error;

    fn try_from(value: &'static Formatting) -> Result<Self, Self::Error> {
        if value.is_color() {
            Ok(Self {
                rgb: value.color_value().unwrap(),
                name: Some(Cow::Borrowed(value.name())),
            })
        } else {
            Err(Error::InvalidColor)
        }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.rgb == other.rgb
    }
}

impl Eq for Color {}

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
    pub fn is_user_definable(self) -> bool {
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
    contents: (Arc<dyn UpdDebug + Send + Sync>, TypeId),
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
            contents: (Arc::new(contents), TypeId::of::<T>()),
            action,
            hash: hasher.finish(),
        }
    }

    #[inline]
    pub fn action(&self) -> &HoverEventAction {
        &self.action
    }

    pub fn value<T: 'static>(&self) -> Option<&T> {
        if TypeId::of::<T>() == self.contents.1 {
            Some(unsafe {
                &*(&*self.contents.0 as *const (dyn UpdDebug + Send + Sync) as *const T)
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

impl PartialEq for HoverEvent {
    fn eq(&self, other: &Self) -> bool {
        self.action() == other.action()
    }
}

impl Eq for HoverEvent {}

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
