use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use rimecraft_primitives::{combine_traits, id, Id};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    lang::{self, Lang},
    Stringified, RGB,
};

use visit::{ErasedVisit, ErasedVisitStyled};

use self::{content::Content, visit::CharVisitor};

use super::fmt::Formatting;

pub mod content;
pub mod visit;

/// An error that can occur when processing a [`Text`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not a valid color")]
    InvalidColor,
    #[error("no valid color value found")]
    ColorValueNotFound,
    #[error("unable to parse integer: {0}")]
    ParseInt(std::num::ParseIntError),
    #[error("formatting error: {0}")]
    Formatting(super::fmt::Error),
    #[error("invalid name: {0}")]
    InvalidName(String),
}

combine_traits! {
    pub trait ErasedContent: ErasedVisit, ErasedVisitStyled, Debug, Send, Sync
}

/// An object that can supply character code points to a
/// visitor, with a style context.
pub trait OrderedText {
    fn accept(&self, visitor: &mut dyn CharVisitor) -> bool;
}

impl<T> OrderedText for Box<T>
where
    T: OrderedText + ?Sized,
{
    #[inline]
    fn accept(&self, visitor: &mut dyn CharVisitor) -> bool {
        self.as_ref().accept(visitor)
    }
}

pub struct Text {
    content: Arc<dyn ErasedContent>,
    sibs: Vec<Self>,
    style: Style,

    /// Hash code of `content`.
    content_hash: u64,
}

impl Text {
    fn new<T: 'static>(content: T, siblings: Vec<Self>, style: Style) -> Self
    where
        T: Content + Hash + Debug + Send + Sync,
    {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);

        Self {
            content: Arc::new(content),
            sibs: siblings,
            style,
            content_hash: hasher.finish(),
        }
    }

    /// Returns the content of this text.
    #[inline]
    pub fn content(&self) -> &dyn ErasedContent {
        &*self.content
    }

    /// Returns the siblings of this text.
    #[inline]
    pub fn siblings(&self) -> &[Self] {
        &self.sibs
    }

    /// Returns the style of this text.
    #[inline]
    pub fn style(&self) -> &Style {
        &self.style
    }

    /// Sets the style of this text.
    #[inline]
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Pushes a text to this text's siblings.
    #[inline]
    pub fn push(&mut self, value: Self) {
        self.sibs.push(value)
    }

    /// Updates the style of this text.
    pub fn styled<F>(&mut self, f: F)
    where
        F: FnOnce(Style) -> Style,
    {
        self.style = f(std::mem::take(&mut self.style))
    }

    /// Fills the absent parts of this text's style with definitions
    /// from the given style.
    pub fn fill_style(&mut self, style_override: Style) {
        self.style = style_override.with_parent(std::mem::take(&mut self.style));
    }

    /// Adds a formatting to this text's style.
    pub fn formatted(&mut self, fmt: Formatting) {
        self.style = std::mem::take(&mut self.style).with_formatting(fmt);
    }

    /// Adds some formattings to this text's style.
    pub fn multi_formatted(&mut self, fmts: &[Formatting]) {
        self.style = std::mem::take(&mut self.style).with_formattings(fmts);
    }

    pub fn to_ordered_text(&self) -> impl OrderedText + '_ {
        lang::global().unwrap().reorder(self)
    }
}

impl<T> From<T> for Text
where
    T: Content + Hash + Debug + Send + Sync + 'static,
{
    /// Creates a piece of mutable text with the given content,
    /// with no sibling and style.
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value, vec![], Style::EMPTY)
    }
}

impl Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("content", &self.content)
            .field("style", &self.style)
            .field("siblings", &self.sibs)
            .finish()
    }
}

impl Hash for Text {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sibs.hash(state);
        self.style.hash(state);
        self.content_hash.hash(state);
    }
}

/// The style of a [`Text`], representing cosmetic attributes.
/// It includes font, color, click event, hover event, etc.
///
/// A style should be immutable.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.Style` (yarn).
#[derive(Clone, PartialEq, Eq, Debug, Default, Deserialize, Hash)]
pub struct Style {
    /// The color of this style.
    #[serde(default)]
    color: Option<Color>,

    /// Whether this style has bold formatting.
    #[serde(default)]
    bold: Option<bool>,

    /// Whether this style has italic formatting.
    #[serde(default)]
    italic: Option<bool>,

    /// Whether this style has underlined formatting.
    #[serde(default)]
    underlined: Option<bool>,

    /// Whether this style has strikethrough formatting.
    #[serde(default)]
    strikethrough: Option<bool>,

    /// Whether this style has obfuscated formatting.
    #[serde(default)]
    obfuscated: Option<bool>,

    /// The click event of this style.
    #[serde(default)]
    click: Option<ClickEvent>,

    /// The hover event of this style.
    #[serde(default)]
    hover: Option<HoverEvent>,

    /// The insertion text of this style.
    #[serde(default)]
    insertion: Option<String>,

    /// The font ID of this style.
    #[serde(default)]
    font: Option<Id>,
}

impl Style {
    /// An empty style.
    pub const EMPTY: Self = Self {
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

    /// Returns the color of this style.
    #[inline]
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// Returns whether this style has bold formatting.
    ///
    /// See [`Formatting::Bold`].
    #[inline]
    pub fn is_bold(&self) -> bool {
        self.bold.unwrap_or(false)
    }

    /// Returns whether this style has italic formatting.
    ///
    /// See [`Formatting::Italic`].
    #[inline]
    pub fn is_italic(&self) -> bool {
        self.italic.unwrap_or(false)
    }

    /// Returns whether this style has strikethrough formatting.
    ///
    /// See [`Formatting::Strikethrough`].
    #[inline]
    pub fn is_strikethrough(&self) -> bool {
        self.strikethrough.unwrap_or(false)
    }

    /// Returns whether this style has underlined formatting.
    ///
    /// See [`Formatting::Underline`].
    #[inline]
    pub fn is_underlined(&self) -> bool {
        self.underlined.unwrap_or(false)
    }

    /// Returns whether this style has obfuscated formatting.
    ///
    /// See [`Formatting::Obfuscated`].
    #[inline]
    pub fn is_obfuscated(&self) -> bool {
        self.obfuscated.unwrap_or(false)
    }

    /// Returns whether this style is empty.
    ///
    /// See [`Self::EMPTY`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self == &Self::EMPTY
    }

    /// Returns the click event of this style.
    #[inline]
    pub fn click_event(&self) -> Option<&ClickEvent> {
        self.click.as_ref()
    }

    /// Returns the hover event of this style.
    #[inline]
    pub fn hover_event(&self) -> Option<&HoverEvent> {
        self.hover.as_ref()
    }

    /// Returns the insertion text of this style.
    ///
    /// An insertion text is a text that is inserted into the chat
    /// when the player shift-clicks on the text.
    #[inline]
    pub fn insertion(&self) -> Option<&String> {
        self.insertion.as_ref()
    }

    /// Returns the font ID of this style.
    pub fn font(&self) -> Cow<'_, Id> {
        self.font
            .as_ref()
            .map_or_else(|| Cow::Owned(id!(Self::DEFAULT_FONT_ID)), Cow::Borrowed)
    }

    /// Returns a new style with given color and all other attributes
    /// of this style.
    #[inline]
    pub fn with_color(self, value: Option<Color>) -> Self {
        Self {
            color: value,
            ..self
        }
    }

    /// Returns a new style with given bold attribute and all other
    /// attributes of this style.
    #[inline]
    pub fn with_bold(self, value: Option<bool>) -> Self {
        Self {
            bold: value,
            ..self
        }
    }

    /// Returns a new style with given italic attribute and all other
    /// attributes of this style.
    #[inline]
    pub fn with_italic(self, value: Option<bool>) -> Self {
        Self {
            italic: value,
            ..self
        }
    }

    /// Returns a new style with given underline attribute and all
    /// other attributes of this style.
    #[inline]
    pub fn with_underline(self, value: Option<bool>) -> Self {
        Self {
            underlined: value,
            ..self
        }
    }

    /// Returns a new style with given strikethrough attribute and
    /// all other attributes of this style.
    #[inline]
    pub fn with_strikethrough(self, value: Option<bool>) -> Self {
        Self {
            strikethrough: value,
            ..self
        }
    }

    /// Returns a new style with given obfuscated attribute and all
    /// other attributes of this style.
    #[inline]
    pub fn with_obfuscated(self, value: Option<bool>) -> Self {
        Self {
            obfuscated: value,
            ..self
        }
    }

    /// Returns a new style with given click event and all other
    /// attributes of this style.
    #[inline]
    pub fn with_click_event(self, value: Option<ClickEvent>) -> Self {
        Self {
            click: value,
            ..self
        }
    }

    /// Returns a new style with given hover event and all other
    /// attributes of this style.
    #[inline]
    pub fn with_hover_event(self, value: Option<HoverEvent>) -> Self {
        Self {
            hover: value,
            ..self
        }
    }

    /// Returns a new style with given insertion and all other
    /// attributes of this style.
    #[inline]
    pub fn with_insertion(self, value: Option<String>) -> Self {
        Self {
            insertion: value,
            ..self
        }
    }

    /// Returns a new style with given font ID and all other
    /// attributes of this style.
    #[inline]
    pub fn with_font(self, value: Option<Id>) -> Self {
        Self {
            font: value,
            ..self
        }
    }

    /// Returns a new style with given formatting and all other
    /// attributes of this style.
    pub fn with_formatting(mut self, value: Formatting) -> Self {
        match value {
            Formatting::Bold => self.bold = Some(true),
            Formatting::Italic => self.italic = Some(true),
            Formatting::Underline => self.underlined = Some(true),
            Formatting::Strikethrough => self.strikethrough = Some(true),
            Formatting::Obfuscated => self.obfuscated = Some(true),
            Formatting::Reset => self = Self::EMPTY,
            _ => {
                if let Ok(color) = Color::try_from(value) {
                    self.color = Some(color)
                }
            }
        }

        self
    }

    /// Returns a new style with given formattings and all other
    /// attributes of this style.
    ///
    /// See [`Self::with_formatting`].
    pub fn with_formattings(mut self, values: &[Formatting]) -> Self {
        for value in values {
            self = self.with_formatting(*value);
        }
        self
    }

    /// Returns a new style with given formatting and some applicable
    /// attributes of this style.
    ///
    /// When a color formatting is passed, the other formattings, including
    /// bold, italic, underlined, strikethrough, and obfuscated, will be
    /// all removed.
    pub fn with_formatting_exclusive(mut self, value: Formatting) -> Self {
        match value {
            Formatting::Bold => self.bold = Some(true),
            Formatting::Italic => self.italic = Some(true),
            Formatting::Underline => self.underlined = Some(true),
            Formatting::Strikethrough => self.strikethrough = Some(true),
            Formatting::Obfuscated => self.obfuscated = Some(true),
            Formatting::Reset => self = Self::EMPTY,
            _ => {
                if let Ok(color) = Color::try_from(value) {
                    self.color = Some(color);

                    self.bold = None;
                    self.italic = None;
                    self.underlined = None;
                    self.strikethrough = None;
                    self.obfuscated = None;
                }
            }
        }

        self
    }

    /// Returns a new style with the undefined attributes of this
    /// style filled by the parent style.
    pub fn with_parent(mut self, value: Self) -> Self {
        if self.is_empty() {
            value
        } else if value.is_empty() {
            self
        } else {
            macro_rules! parent {
                ($($f:ident),*) => {
                    $(if self.$f.is_none() {
                        self.$f = value.$f;
                    })*
                    self
                };
            }

            parent! {
                color,
                bold,
                italic,
                underlined,
                strikethrough,
                obfuscated,
                click,
                hover,
                insertion,
                font
            }
        }
    }
}

impl Serialize for Style {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Don't serialize empty attributes.

        macro_rules! serialize {
            ($($f:ident),*) => {
                {
                    use serde::ser::SerializeStruct;
                    let mut count = 0;
                    $(if self.$f.is_some() { count += 1; })*
                    let mut state = serializer.serialize_struct("Style", count)?;
                    $(if let Some(value) = &self.$f { state.serialize_field(stringify!($f), value)?; })*
                    state.end()
                }
            };
        }

        serialize! {
            color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
            click,
            hover,
            insertion,
            font
        }
    }
}

/// Represents an action that should be performed when the text is clicked.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.ClickEvent` (yarn).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
pub enum ClickEvent {
    /// Opens the given URL.
    OpenUrl(Url),
    /// Opens the given file.
    OpenFile(PathBuf),
    /// Runs the given command.
    RunCommand(String),
    /// Suggests the given command.
    SuggestCommand(String),
    /// Changes the page to the given page.
    ChangePage(Stringified<u32>),
    /// Copies the given text to the clipboard.
    CopyToClipboard(String),
}

impl ClickEvent {
    #[inline]
    pub fn is_user_definable(self) -> bool {
        !matches!(self, Self::OpenFile(_))
    }
}

/// Represents an action that should be performed when the text is hovered.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.HoverEvent` (yarn).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "contents")]
pub enum HoverEvent {
    ShowText,
    ShowItem,
    ShowEntity,
}

/// Represents an RGB color of a [`Text`].
///
/// This is immutable as a part of [`Style`].
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.TextColor` (yarn).
#[derive(Clone, Copy, Debug, Eq)]
pub struct Color {
    /// A 24-bit color.
    rgb: RGB,
    name: Option<&'static str>,
}

impl Color {
    const RGB_PREFIX: &str = "#";

    /// Returns the inner RGB value of this color.
    #[inline]
    pub fn rgb(self) -> RGB {
        self.rgb
    }

    /// Returns the hex code of this color.
    #[inline]
    fn to_hex_code(self) -> String {
        format!("{}{:06X}", Self::RGB_PREFIX, self.rgb)
    }

    /// Returns the name of this color.
    pub fn name(self) -> Cow<'static, str> {
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

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
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
