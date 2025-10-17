//! Iterators over text items with associated [`Style`]s.

use rimecraft_fmt::Formatting;

use crate::Style;

/// An item in an iterator over text items with associated [`Style`]s.
///
/// See: [`IterText`]
#[derive(Debug, Clone)]
pub struct IterTextItem<StyleExt> {
    /// The index of the character in the original string.
    pub index: usize,
    /// The character.
    pub c: char,
    /// The [`Style`] associated with the character.
    pub style: Style<StyleExt>,
}

/// An iterator over text items with associated [`Style`]s. Based on indexed [`char`]s.
pub trait IterText<StyleExt> {
    /// Returns an [`Iterator`] over text items with associated [`Style`]s, whose items are of type [`IterTextItem`].
    fn iter_text(&self) -> impl Iterator<Item = IterTextItem<StyleExt>> + '_;
}

impl<T, StyleExt> IterText<StyleExt> for T
where
    T: Iterator<Item = IterTextItem<StyleExt>> + Clone,
{
    fn iter_text(&self) -> impl Iterator<Item = IterTextItem<StyleExt>> + '_ {
        self.clone()
    }
}

/// Returns an empty [`IterText`] over text items with associated [`Style`]s.
pub fn empty<StyleExt: 'static>() -> impl IterText<StyleExt> {
    std::iter::empty()
}

/// Returns a single-character [`IterText`] with the given [`char`] and [`Style`].
pub fn styled_char<StyleExt>(c: char, style: Style<StyleExt>) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    std::iter::once(IterTextItem { index: 0, c, style })
}

/// Returns a forward [`IterText`] over the characters of the given string with the given [`Style`].
pub fn styled_str<StyleExt>(s: &str, style: Style<StyleExt>) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    s.chars().enumerate().map(move |(i, c)| IterTextItem {
        index: i,
        c,
        style: style.clone(),
    })
}

/// Returns a backward [`IterText`] over the characters of the given string with the given [`Style`].
pub fn styled_str_rev<StyleExt>(s: &str, style: Style<StyleExt>) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    s.chars().rev().enumerate().map(move |(i, c)| IterTextItem {
        index: i,
        c,
        style: style.clone(),
    })
}

/// Returns an [`IterText`] over the characters of the given formatted string,
/// starting from the given index, applying formatting codes as specified,
/// with the given starting and reset [`Style`]s.
pub fn iter_formatted<StyleExt>(
    str: &str,
    start_index: usize,
    starting_style: Style<StyleExt>,
    reset_style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
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
            return Some(IterTextItem {
                index: chars.clone().count() + start_index - 1,
                c,
                style: style.clone(),
            });
        }
        None
    })
}

/// Returns an [`IterText`] over the characters of the given formatted string,
/// starting from the given index, applying formatting codes as specified,
/// with a unified [`Style`] for starting and reset.
pub fn iter_formatted_unified<StyleExt>(
    str: &str,
    start_index: usize,
    style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    iter_formatted(str, start_index, style.clone(), style)
}

/// Returns an [`IterText`] over the characters of the given formatted string,
/// starting from index `0`, applying formatting codes as specified,
/// with a unified [`Style`] for starting and reset.
pub fn iter_formatted_unified_from_start<StyleExt>(
    str: &str,
    style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    iter_formatted_unified(str, 0, style)
}

/// Removes formatting codes from the given string and returns the plain text.
///
/// See: [`iter_formatted_unified_from_start`]
pub fn remove_formatting_codes(str: &str) -> String {
    let iter = iter_formatted_unified_from_start(str, Style::default());
    iter.iter_text()
        .map(
            |IterTextItem::<()> {
                 index: _,
                 c,
                 style: _,
             }| c,
        )
        .collect()
}
