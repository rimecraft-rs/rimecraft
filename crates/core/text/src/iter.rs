//! Iterator types for text processing.

use std::{fmt::Debug, ops::Add};

use crate::style::Style;

/// An iterator over the content and style of a text.
pub struct StyledIter<'a, T, StyleExt> {
    pub(crate) style: &'a Style<StyleExt>,
    pub(crate) inner: Box<dyn Iterator<Item = (&'a T, Style<StyleExt>)> + 'a>,
}

impl<'a, T, StyleExt> Iterator for StyledIter<'a, T, StyleExt>
where
    StyleExt: Add<Output = StyleExt> + Clone,
{
    type Item = (&'a T, Style<StyleExt>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (content, style) = self.inner.next()?;
        Some((content, self.style.clone() + style))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T, StyleExt> Debug for StyledIter<'_, T, StyleExt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StyledIter").finish()
    }
}

/// An iterator over the content of a text.
pub struct Iter<'a, T> {
    pub(crate) inner: Box<dyn Iterator<Item = &'a T> + 'a>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T> Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter").finish()
    }
}
