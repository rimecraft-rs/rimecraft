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
use rimecraft_primitives::{id, ErasedSerDeUpdate, Id, SerDeUpdate};

use crate::Rgb;

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
    #[error("invalid name: {0}")]
    InvalidName(String),
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
#[derive(PartialEq)]
pub struct Style {
    pub color: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub click: Option<ClickEvent>,
    pub hover: Option<HoverEvent>,
    pub insertion: Option<String>,
    pub font: Option<Id>,
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
#[derive(Debug, Eq)]
pub struct Color {
    /// A 24-bit color.
    rgb: Rgb,
    name: Option<&'static str>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    #[inline]
    pub fn rgb(&self) -> Rgb {
        self.rgb
    }

    #[inline]
    fn to_hex_code(&self) -> String {
        format!("{}{:06X}", Self::RGB_PREFIX, self.rgb)
    }

    pub fn name(&self) -> Cow<'static, str> {
        self.name
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(self.to_hex_code()))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name {
            f.write_str(name)
        } else {
            f.write_str(&self.to_hex_code())
        }
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(value) = s.strip_prefix(Self::RGB_PREFIX) {
            Ok(Self {
                rgb: value.parse().map_err(Error::ParseInt)?,
                name: None,
            })
        } else {
            s.parse::<Formatting>()
                .map_err(Error::Formatting)?
                .try_into()
        }
    }
}

impl TryFrom<Formatting> for Color {
    type Error = Error;

    fn try_from(value: Formatting) -> Result<Self, Self::Error> {
        Ok(Self {
            rgb: value.color_value().ok_or(Error::ColorValueNotFound)?,
            name: Some(value.name()),
        })
    }
}

impl PartialEq for Color {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.rgb == other.rgb
    }
}

impl Hash for Color {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rgb.hash(state);
        self.name.hash(state)
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct ClickEvent {
    action: ClickEventAction,
    value: String,
}

impl ClickEvent {
    #[inline]
    pub fn new(action: ClickEventAction, value: String) -> Self {
        Self { action, value }
    }

    #[inline]
    pub fn action(&self) -> ClickEventAction {
        self.action
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.value
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
}

impl FromStr for ClickEventAction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static MAP: Lazy<HashMap<String, ClickEventAction>> = Lazy::new(|| {
            ClickEventAction::VALUES
                .into_iter()
                .map(|value| (value.name().to_owned(), value))
                .collect()
        });

        MAP.get(s)
            .copied()
            .ok_or_else(|| Error::InvalidName(s.to_owned()))
    }
}

trait UpdDebug: ErasedSerDeUpdate + Debug {}
impl<T> UpdDebug for T where T: ?Sized + ErasedSerDeUpdate + Debug {}

erased_serde::serialize_trait_object!(UpdDebug);
rimecraft_primitives::update_trait_object!(UpdDebug);

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

impl Serialize for HoverEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Struct<'a> {
            action: &'static HoverEventAction,
            contents: &'a (dyn UpdDebug + Send + Sync),
        }

        Struct {
            action: self.action,
            contents: &*self.contents.0,
        }
        .serialize(serializer)
    }
}

impl SerDeUpdate for HoverEvent {
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Action,
            Contents,
        }

        struct HEVisitor;

        impl<'de> serde::de::Visitor<'de> for HEVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct HoverEvent { action, contents }")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                todo!()
            }
        }
        todo!()
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
        f.debug_struct("HoverEventAction")
            .field("name", &self.name)
            .finish()
    }
}

impl Serialize for HoverEventAction {
    #[inline]
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
