//! Iterators over text items with associated [`Style`]s.

use std::fmt::Debug;

use rimecraft_fmt::Formatting;

use crate::{ProvideTextTy, Style, style::Formattable};

/// An item in an iterator over text items with associated [`Style`]s.
///
/// See: [`OrderedText`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
pub struct OrderedText<'a, Cx>(Box<dyn Iterator<Item = OrderedTextItem<Cx>> + 'a>)
where
    Cx: ProvideTextTy;

impl<Cx> Debug for OrderedText<'_, Cx>
where
    Cx: ProvideTextTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedText").finish()
    }
}

impl<Cx> Iterator for OrderedText<'_, Cx>
where
    Cx: ProvideTextTy,
{
    type Item = OrderedTextItem<Cx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, Cx> OrderedText<'a, Cx>
where
    Cx: ProvideTextTy,
{
    #[inline]
    fn of<T>(value: T) -> Self
    where
        T: IntoIterator<Item = OrderedTextItem<Cx>> + 'a,
    {
        Self(Box::new(value.into_iter()))
    }
}

impl<'a, Cx> OrderedText<'a, Cx>
where
    Cx: ProvideTextTy,
{
    /// Returns an empty [`OrderedText`].
    pub fn empty() -> Self {
        Self::of(std::iter::empty())
    }

    /// Returns an [`OrderedText`] with a single styled character.
    pub fn styled_char(c: char, style: Style<Cx::StyleExt>) -> Self {
        Self::of(std::iter::once(OrderedTextItem { index: 0, c, style }))
    }

    /// Returns an [`OrderedText`] over the characters of the given string,
    /// all with the given [`Style`].
    pub fn styled_str(s: &'a str, style: Style<Cx::StyleExt>) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        Self::of(s.chars().enumerate().map(move |(i, c)| OrderedTextItem {
            index: i,
            c,
            style: style.clone(),
        }))
    }

    /// Returns an [`OrderedText`] over the characters of the given string in reverse order,
    /// all with the given [`Style`].
    pub fn styled_str_rev(s: &'a str, style: Style<Cx::StyleExt>) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        Self::of(
            s.chars()
                .rev()
                .enumerate()
                .map(move |(i, c)| OrderedTextItem {
                    index: i,
                    c,
                    style: style.clone(),
                }),
        )
    }

    /// Returns an [`OrderedText`] over the characters of the given formatted string,
    /// starting from the given index, applying formatting codes as specified.
    pub fn iter_formatted(
        str: &'a str,
        start_index: usize,
        starting_style: Style<Cx::StyleExt>,
        reset_style: Style<Cx::StyleExt>,
    ) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        let mut style = starting_style;
        let mut chars = str.chars().enumerate().skip(start_index).peekable();

        Self::of(std::iter::from_fn(move || {
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
        }))
    }

    /// Returns an [`OrderedText`] over the characters of the given formatted string,
    /// starting from the given index, applying formatting codes as specified,
    /// with a unified [`Style`] for starting and reset.
    pub fn iter_formatted_unified(
        str: &'a str,
        start_index: usize,
        style: Style<Cx::StyleExt>,
    ) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        Self::iter_formatted(str, start_index, style.clone(), style)
    }

    /// Returns an [`OrderedText`] over the characters of the given formatted string,
    /// applying formatting codes as specified,
    /// with a unified [`Style`] for starting and reset.
    pub fn iter_formatted_unified_from_start(str: &'a str, style: Style<Cx::StyleExt>) -> Self
    where
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        Self::iter_formatted_unified(str, 0, style)
    }

    /// Removes formatting codes from the given string and returns the plain text.
    ///
    /// See: [`Self::iter_formatted_unified_from_start`]
    pub fn remove_formatting_codes(str: &'a str) -> String
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
