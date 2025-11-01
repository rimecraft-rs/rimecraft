//! Minecraft text API.

#[cfg(feature = "macros")]
mod macros;

#[doc(hidden)]
pub mod __priv_macro_use {
    pub use std::concat;
    pub use std::string::{String, ToString};
    pub use std::vec::Vec;

    #[inline]
    pub const fn strip_dot_prefix(s: &str) -> &str {
        assert!(s.len() >= ".".len());
        __strip_dot_prefix(s)
    }

    const fn __strip_dot_prefix(s: &str) -> &str {
        let prefix_len = ".".len();
        let bytes = s.as_bytes();
        let len = bytes.len() - prefix_len;
        let ptr = bytes.as_ptr();
        let bytes = unsafe { std::slice::from_raw_parts(ptr.add(prefix_len), len) };
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

use remap::{remap, remap_method};

#[cfg(feature = "macros")]
pub use rimecraft_text_derive::Localize;

mod error;
mod iter;
pub mod ordered;
pub mod style;

#[cfg(feature = "serde")]
mod _serde;

use std::{borrow::Cow, fmt::Display, ops::Add};

use rimecraft_global_cx::GlobalContext;

use crate::style::Formattable;

pub use error::Error;
pub use iter::{Iter, StyledIter};
pub use ordered::{ErasedOrderedText, OrderedText};
pub use style::Style;

/// A raw text component.
///
/// Each text has a tree structure, embodying all its siblings.
/// See [`Self::sibs`].
///
/// # Serialize and Deserialize
///
/// This type can be serialized and deserialized using the `serde` feature.
/// Serialized raw format could be checked on the [Minecraft Wiki](https://minecraft.wiki/w/Raw_JSON_text_format).
///
/// _The type `T` (content) should implement `serde::Serialize` and `serde::Deserialize`,
/// and the variant type (the `type` field) should be an optional field, as the same as the
/// format in Java Edition._
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawText<T, StyleExt> {
    content: T,
    style: Style<StyleExt>,
    sibs: Vec<Self>,
}

impl<T, StyleExt> RawText<T, StyleExt> {
    /// Creates a new text with the given content and style.
    ///
    /// See [`Self::with_sibs`] for creating a text with siblings.
    #[inline]
    pub const fn new(content: T, style: Style<StyleExt>) -> Self {
        Self {
            content,
            style,
            sibs: Vec::new(),
        }
    }

    /// Creates a new text with the given content, style and siblings.
    #[inline]
    pub const fn with_sibs(content: T, style: Style<StyleExt>, sibs: Vec<Self>) -> Self {
        Self {
            content,
            style,
            sibs,
        }
    }

    /// Returns the style of this text.
    #[inline]
    pub fn style(&self) -> &Style<StyleExt> {
        &self.style
    }

    /// Returns the style of this text.
    #[inline]
    pub fn style_mut(&mut self) -> &mut Style<StyleExt> {
        &mut self.style
    }

    /// Returns the content of this text.
    #[inline]
    #[remap_method(yarn = "getContent", mojmaps = "getContents")]
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Returns the content of this text.
    #[inline]
    #[remap_method(yarn = "getContentMut", mojmaps = "getContentsMut")]
    pub fn content_mut(&mut self) -> &mut T {
        &mut self.content
    }

    /// Returns the siblings of this text.
    #[inline]
    #[remap_method(yarn = "getSiblings", mojmaps = "getSiblings")]
    pub fn sibs(&self) -> &[Self] {
        &self.sibs
    }

    /// Returns the siblings of this text.
    #[inline]
    #[remap_method(yarn = "getSiblingsMut", mojmaps = "getSiblingsMut")]
    pub fn sibs_mut(&mut self) -> &mut Vec<Self> {
        &mut self.sibs
    }

    /// Returns the style of this text.
    #[inline]
    #[remap_method(yarn = "append", mojmaps = "append")]
    pub fn push(&mut self, text: Self) {
        self.sibs.push(text);
    }

    /// Returns an iterator over the content of this text.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T, StyleExt> {
        Iter {
            content: Some(self.content()),
            sibs: self.sibs().iter(),
            sib_iter: None,
        }
    }

    /// Returns an iterator over the content and style of this text.
    #[inline]
    pub fn styled_iter<'a>(&'a self) -> StyledIter<'a, T, StyleExt>
    where
        StyleExt: Add<&'a StyleExt, Output = StyleExt>,
    {
        StyledIter {
            style: self.style(),
            content: Some(self.content()),
            sibs: self.sibs().iter(),
            sib_iter: None,
        }
    }
}

impl<T, StyleExt> RawText<T, StyleExt>
where
    T: Display,
{
    /// Visits the string literals of this text.
    #[remap_method(yarn = "visit", mojmaps = "visit")]
    #[deprecated = "use `Self::iter` instead"]
    pub fn visit<V, U>(&self, mut visitor: V) -> Option<U>
    where
        V: FnMut(&str) -> Option<U>,
    {
        for content in self.iter() {
            if let Some(result) = visitor(&content.to_string()) {
                return Some(result);
            }
        }
        None
    }

    /// Visits the string literals of this text with style.
    #[deprecated = "use `Self::styled_iter` instead"]
    pub fn styled_visit<'a, V, U>(&'a self, mut visitor: V) -> Option<U>
    where
        V: FnMut(Style<StyleExt>, &str) -> Option<U>,
        StyleExt: Add<&'a StyleExt, Output = StyleExt> + Default,
    {
        for (content, style) in self.styled_iter() {
            if let Some(result) = visitor(style, &content.to_string()) {
                return Some(result);
            }
        }
        None
    }
}

impl<T, StyleExt> RawText<T, StyleExt>
where
    T: Display,
{
    /// Visits the string literals of this text.
    #[remap_method(yarn = "visit", mojmaps = "visit")]
    #[deprecated = "use `Self::iter` instead"]
    pub fn visit<V, U>(&self, mut visitor: V) -> Option<U>
    where
        V: FnMut(&str) -> Option<U>,
    {
        for content in self.iter() {
            if let Some(result) = visitor(&content.to_string()) {
                return Some(result);
            }
        }
        None
    }
}

impl<T, StyleExt> RawText<T, StyleExt>
where
    T: Display,
    StyleExt: Add<Output = StyleExt> + Clone,
{
    /// Visits the string literals of this text with style.
    #[deprecated = "use `Self::styled_iter` instead"]
    pub fn styled_visit<V, U>(&self, mut visitor: V) -> Option<U>
    where
        V: FnMut(Style<StyleExt>, &str) -> Option<U>,
    {
        for (content, style) in self.styled_iter() {
            if let Some(result) = visitor(style, &content.to_string()) {
                return Some(result);
            }
        }
        None
    }
}

impl<'a, T, StyleExt> IntoIterator for &'a RawText<T, StyleExt> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, StyleExt>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, StyleExt> From<T> for RawText<T, StyleExt>
where
    StyleExt: Default,
{
    #[inline]
    fn from(content: T) -> Self {
        Self {
            content,
            style: Style::default(),
            sibs: Vec::new(),
        }
    }
}

impl<T, StyleExt> Display for RawText<T, StyleExt>
where
    T: Display,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for content in self {
            write!(f, "{content}")?;
        }
        Ok(())
    }
}

/// Text content that supports literal text conversion.
pub trait Plain: Sized {
    /// Converts a string literal to the text content.
    fn from_literal(literal: &str) -> Self;
}

impl<T> Plain for T
where
    T: for<'a> From<&'a str>,
{
    #[inline]
    fn from_literal(literal: &str) -> Self {
        literal.into()
    }
}

/// Global context for [`Text`].
///
/// The associated type `Content` and `StyleExt` should be applied to [`Text`] when used.
pub trait ProvideTextTy: GlobalContext {
    /// Generic `T` that should be applied to [`Text`].
    type Content: Plain;

    /// Generic `StyleExt` that should be applied to [`Text`].
    type StyleExt: Formattable;
}

/// Context type decorated [`RawText`].
#[remap(yarn = "Text", mojmaps = "Component")]
#[remap(yarn = "MutableText", mojmaps = "MutableComponent")] // two in one
pub type Text<Cx> = RawText<<Cx as ProvideTextTy>::Content, <Cx as ProvideTextTy>::StyleExt>;

/// A localizable value.
pub trait Localize {
    /// Returns the localization key of this value.
    fn localization_key(&self) -> Cow<'_, str>;
}

/// A seed for encoding and decoding [`Text`] through `edcode2` crate.
#[cfg(feature = "edcode")]
pub type EdcodeSeed<Cx> = rimecraft_global_cx::edcode::Nbt<Text<Cx>, Cx>;

#[cfg(test)]
mod tests;
