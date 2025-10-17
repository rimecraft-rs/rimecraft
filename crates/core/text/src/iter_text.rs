use crate::{Style, iter_text};

pub trait IterText<T> {
    fn iter_text(&self) -> impl Iterator<Item = (char, T)> + '_;
}

pub fn empty() -> impl IterText<()> {
    struct Impl;

    impl IterText<()> for Impl {
        fn iter_text(&self) -> impl Iterator<Item = (char, ())> + '_ {
            std::iter::empty()
        }
    }

    Impl
}

pub fn styled<StyleExt>(c: char, style: Style<StyleExt>) -> impl IterText<StyleExt>
where
    StyleExt: Clone,
{
    iter_text! {
        <StyleExt> where StyleExt: Clone;
        (c: char, style: Style<StyleExt>) => {
            std::iter::once((c.to_owned(), style.ext.clone()))
        }
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
        (s: String, style: Style<StyleExt>) => {
            s.chars().map(move |c| (c, style.ext.clone()))
        }
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
        (s: String, style: Style<StyleExt>) => {
            s.chars().rev().map(move |c| (c, style.ext.clone()))
        }
    }
}
