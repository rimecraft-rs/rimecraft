use rimecraft_fmt::Formatting;

use crate::{Style, iter_text};

pub struct IterTextItem<StyleExt> {
    pub index: usize,
    pub c: char,
    pub style: Style<StyleExt>,
}

pub trait IterText<StyleExt> {
    fn iter_text(&self) -> impl Iterator<Item = IterTextItem<StyleExt>> + '_;
}

pub fn empty<StyleExt: 'static>() -> impl IterText<StyleExt> {
    iter_text! {
        <StyleExt> where StyleExt: 'static;
        () -> StyleExt;
        std::iter::empty()
    }
}

pub fn styled<StyleExt>(c: char, style: Style<StyleExt>) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    iter_text! {
        <StyleExt> where StyleExt: Clone;
        (c: char, style: Style<StyleExt>) -> StyleExt;
        std::iter::once(IterTextItem {
            index: 0,
            c: c.to_owned(),
            style: style.to_owned(),
        })
    }
}

pub fn styled_forwards_visited_string<StyleExt>(
    s: &str,
    style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    let s = s.to_owned();

    iter_text! {
        <StyleExt> where StyleExt: Clone;
        (s: String, style: Style<StyleExt>) -> StyleExt;
        s.chars().enumerate().map(move |(i, c)| IterTextItem {
            index: i,
            c,
            style: style.clone(),
        })
    }
}

pub fn styled_backwards_visited_string<StyleExt>(
    s: &str,
    style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    let s = s.to_owned();

    iter_text! {
        <StyleExt> where StyleExt: Clone;
        (s: String, style: Style<StyleExt>) -> StyleExt;
        s.chars().rev().enumerate().map(move |(i, c)| IterTextItem {
            index: i,
            c,
            style: style.clone(),
        })
    }
}

pub fn formatted<StyleExt>(
    str: &str,
    start_index: usize,
    starting_style: Style<StyleExt>,
    reset_style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    let s = str.to_owned();

    iter_text! {
        <StyleExt> where StyleExt: Clone;
        (s: String, start_index: usize, starting_style: Style<StyleExt>, reset_style: Style<StyleExt>) -> StyleExt;
        format(s, start_index.to_owned(), starting_style.to_owned(), reset_style.to_owned())
    }
}

pub fn formatted_unified<StyleExt>(
    str: &str,
    start_index: usize,
    style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    formatted(str, start_index, style.clone(), style)
}

pub fn formatted_unified_from_start<StyleExt>(
    str: &str,
    style: Style<StyleExt>,
) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    formatted_unified(str, 0, style)
}

fn format<StyleExt>(
    str: &str,
    start_index: usize,
    starting_style: Style<StyleExt>,
    reset_style: Style<StyleExt>,
) -> impl Iterator<Item = IterTextItem<StyleExt>>
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
