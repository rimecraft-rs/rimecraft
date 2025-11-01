//! Iterator types for text processing.

use std::{fmt::Debug, iter::FusedIterator, ops::Add};

use crate::{RawText, style::Style};

/// An iterator over the content and style of a text.
pub struct StyledIter<'a, T, StyleExt> {
    pub(crate) style: &'a Style<StyleExt>,
    pub(crate) content: Option<&'a T>,
    pub(crate) sibs: std::slice::Iter<'a, RawText<T, StyleExt>>,
    pub(crate) sib_iter: Option<Box<Self>>,
}

impl<'a, T, StyleExt> FusedIterator for StyledIter<'a, T, StyleExt> where
    StyleExt: Add<&'a StyleExt, Output = StyleExt> + Default
{
}

impl<'a, T, StyleExt> Iterator for StyledIter<'a, T, StyleExt>
where
    StyleExt: Add<&'a StyleExt, Output = StyleExt> + Default,
{
    type Item = (&'a T, Style<StyleExt>);

    fn next(&mut self) -> Option<Self::Item> {
        self.content
            .take()
            .map(|c| (c, Default::default()))
            .or_else(|| self.sib_iter.as_deref_mut().and_then(Iterator::next))
            .or_else(|| {
                let mut sib_iter = self.sibs.next()?.styled_iter();
                let item = sib_iter.next();
                self.sib_iter = Some(Box::new(sib_iter));
                item
            })
            .map(|(c, s)| (c, s + self.style))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let floor = (self.content.is_some() as usize)
            + self.sibs.size_hint().0
            + self.sib_iter.as_deref().map_or(0, |i| i.size_hint().0);
        (floor, None)
    }
}

impl<T, StyleExt> Debug for StyledIter<'_, T, StyleExt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StyledIter").finish_non_exhaustive()
    }
}

/// An iterator over the content of a text.
pub struct Iter<'a, T, StyleExt> {
    pub(crate) content: Option<&'a T>,
    pub(crate) sibs: std::slice::Iter<'a, RawText<T, StyleExt>>,
    pub(crate) sib_iter: Option<Box<Self>>,
}

impl<T, StyleExt> FusedIterator for Iter<'_, T, StyleExt> {}

impl<'a, T, StyleExt> Iterator for Iter<'a, T, StyleExt> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.content
            .take()
            .or_else(|| self.sib_iter.as_deref_mut().and_then(Iterator::next))
            .or_else(|| {
                let mut sib_iter = self.sibs.next()?.iter();
                let item = sib_iter.next();
                self.sib_iter = Some(Box::new(sib_iter));
                item
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let floor = (self.content.is_some() as usize)
            + self.sibs.size_hint().0
            + self.sib_iter.as_deref().map_or(0, |i| i.size_hint().0);
        (floor, None)
    }
}

impl<T, StyleExt> Debug for Iter<'_, T, StyleExt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter").finish_non_exhaustive()
    }
}
