//! Iterators over text items with associated [`Style`]s.

use std::fmt::Debug;

use rimecraft_fmt::Formatting;

use crate::{ProvideTextTy, Style, style::Formattable};

/// An item in an iterator over text items with associated [`Style`]s.
///
/// See: [`OrderedText`]
#[derive(Debug, Clone)]
pub struct OrderedTextItem<Cx>
where
    Cx: ProvideTextTy,
{
    /// The index of the character in the original string.
    pub index: usize,
    /// The character.
    pub c: char,
    /// The [`Style`] associated with the character.
    pub style: Style<Cx::StyleExt>,
}

/// An iterator over indexed [`char`]s with associated [`Style`]s.
pub struct OrderedText<Cx>(Box<dyn Iterator<Item = OrderedTextItem<Cx>>>)
where
    Cx: ProvideTextTy;

impl<T, Cx> From<T> for OrderedText<Cx>
where
    Cx: ProvideTextTy,
    T: IntoIterator<Item = OrderedTextItem<Cx>>,
{
    #[inline]
    fn from(value: T) -> Self {
        let vec: Vec<OrderedTextItem<Cx>> = value.into_iter().collect();
        Self(Box::new(vec.into_iter()))
    }
}

impl<Cx> Debug for OrderedText<Cx>
where
    Cx: ProvideTextTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedText").finish()
    }
}

impl<Cx> OrderedText<Cx>
where
    Cx: ProvideTextTy,
{
    /// Returns an empty [`OrderedText`].
    pub fn empty() -> Self {
        std::iter::empty().into()
    }

    /// Returns an [`OrderedText`] with a single styled character.
    pub fn styled_char(c: char, style: Style<Cx::StyleExt>) -> Self {
        std::iter::once(OrderedTextItem { index: 0, c, style }).into()
    }

    /// Returns an [`OrderedText`] over the characters of the given string,
    /// all with the given [`Style`].
    pub fn styled_str(s: &str, style: Style<Cx::StyleExt>) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        s.chars()
            .enumerate()
            .map(move |(i, c)| OrderedTextItem {
                index: i,
                c,
                style: style.clone(),
            })
            .into()
    }

    /// Returns an [`OrderedText`] over the characters of the given string in reverse order,
    /// all with the given [`Style`].
    pub fn styled_str_rev(s: &str, style: Style<Cx::StyleExt>) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        s.chars()
            .rev()
            .enumerate()
            .map(move |(i, c)| OrderedTextItem {
                index: i,
                c,
                style: style.clone(),
            })
            .into()
    }

    /// Returns an [`OrderedText`] over the characters of the given formatted string,
    /// starting from the given index, applying formatting codes as specified.
    pub fn iter_formatted(
        str: &str,
        start_index: usize,
        starting_style: Style<Cx::StyleExt>,
        reset_style: Style<Cx::StyleExt>,
    ) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        let mut style = starting_style;
        let mut chars = str.chars().enumerate().skip(start_index).peekable();

        std::iter::from_fn(move || {
            while let Some((_, c)) = chars.next() {
                // Checks for formatting code prefix (§ = U+00A7)
                if c == Formatting::CODE_PREFIX {
                    // Peeks at the next character to see if it's a valid formatting code
                    if let Some(&(_, code_char)) = chars.peek()
                        && let Ok(formatting) = char::try_into(code_char)
                    {
                        // Valid formatting code found
                        style = if formatting == Formatting::Reset {
                            reset_style.clone()
                        } else {
                            style.clone().with_exclusive_formatting(formatting)
                        };
                        // Skips the code character
                        chars.next();
                        continue;
                    }
                    // If we reach here, '§' was at the end or followed by invalid code
                    // Breaks out of the loop
                    break;
                }

                // Regular character - yield it
                return Some(OrderedTextItem {
                    index: chars.clone().count() + start_index - 1,
                    c,
                    style: style.clone(),
                });
            }
            None
        })
        .into()
    }

    /// Returns an [`OrderedText`] over the characters of the given formatted string,
    /// starting from the given index, applying formatting codes as specified,
    /// with a unified [`Style`] for starting and reset.
    pub fn iter_formatted_unified(str: &str, start_index: usize, style: Style<Cx::StyleExt>) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        Self::iter_formatted(str, start_index, style.clone(), style)
    }

    /// Returns an [`OrderedText`] over the characters of the given formatted string,
    /// applying formatting codes as specified,
    /// with a unified [`Style`] for starting and reset.
    pub fn iter_formatted_unified_from_start(str: &str, style: Style<Cx::StyleExt>) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        Self::iter_formatted_unified(str, 0, style)
    }

    /// Removes formatting codes from the given string and returns the plain text.
    ///
    /// See: [`Self::iter_formatted_unified_from_start`]
    pub fn remove_formatting_codes(str: &str) -> String
    where
        <Cx as ProvideTextTy>::StyleExt: Clone + Default,
    {
        let iter = Self::iter_formatted_unified_from_start(str, Style::default());
        iter.0
            .map(
                |OrderedTextItem::<Cx> {
                     index: _,
                     c,
                     style: _,
                 }| c,
            )
            .collect()
    }
}
