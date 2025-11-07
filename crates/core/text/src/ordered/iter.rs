use std::{fmt::Debug, iter::FusedIterator, pin::Pin, sync::Arc};

use remap::remap;

use crate::{ProvideTextTy, Style, ordered::OrderedTextItem};

/// Iterator type returned by [`iter_forwards`].
pub struct IterForwards<'a, Cx>
where
    Cx: ProvideTextTy,
{
    chars: std::str::CharIndices<'a>,
    style: Style<Cx::StyleExt>,
}

/// Creates an iterator over the characters of a string slice with given [`Style`]s attached.
///
/// The returned iterator is a [`DoubleEndedIterator`] which means you can make it go backwards.
///
/// See [`iter_forwards_owned`] for an owned version.
#[inline]
#[remap(yarn = "visitForwards", mojmaps = "iterate")]
pub fn iter_forwards<Cx>(text: &str, style: Style<Cx::StyleExt>) -> IterForwards<'_, Cx>
where
    Cx: ProvideTextTy,
{
    IterForwards {
        chars: text.char_indices(),
        style,
    }
}

/// Iterator type returned by [`iter_backwards`].
pub struct IterBackwards<'a, Cx>
where
    Cx: ProvideTextTy,
{
    inner: IterForwards<'a, Cx>,
}

/// [`iter_forwards`] but in reverse order.
///
/// This is a shorthand for `iter_forwards(text, style).rev()`.
#[inline]
#[remap(yarn = "visitBackwards", mojmaps = "iterateBackwards")]
pub fn iter_backwards<Cx>(text: &str, style: Style<Cx::StyleExt>) -> IterBackwards<'_, Cx>
where
    Cx: ProvideTextTy,
{
    IterBackwards {
        inner: iter_forwards(text, style),
    }
}

/// Iterator type returned by [`iter_forwards_owned`].
pub struct IterForwardsOwned<Cx>
where
    Cx: ProvideTextTy,
{
    // SAFETY for self-referencing iterators:
    // - `_str` is pinned semantically for access of `chars`.
    // - `_str`'s address is stable between clones.
    inner: IterForwards<'static, Cx>,
    _str: Pin<Arc<str>>,
}

/// Creates an iterator over the characters of an owned string with given [`Style`]s attached.
///
/// The returned iterator is a [`DoubleEndedIterator`] which means you can make it go backwards.
#[remap(yarn = "visitForwardsOwned", mojmaps = "iterateOwned")]
pub fn iter_forwards_owned<Cx>(text: String, style: Style<Cx::StyleExt>) -> IterForwardsOwned<Cx>
where
    Cx: ProvideTextTy,
{
    let pinned = Pin::new(Arc::from(text.into_boxed_str()));
    IterForwardsOwned {
        // SAFETY: see above.
        inner: iter_forwards(unsafe { &*std::ptr::from_ref(&pinned) }, style),
        _str: pinned,
    }
}

/// Iterator type returned by [`iter_backwards_owned`].
pub struct IterBackwardsOwned<Cx>
where
    Cx: ProvideTextTy,
{
    inner: IterForwardsOwned<Cx>,
}

/// [`iter_forwards_owned`] but in reverse order.
///
/// This is a shorthand for `iter_forwards_owned(text, style).rev()`.
#[inline]
#[remap(yarn = "visitBackwardsOwned", mojmaps = "iterateBackwardsOwned")]
pub fn iter_backwards_owned<Cx>(text: String, style: Style<Cx::StyleExt>) -> IterBackwardsOwned<Cx>
where
    Cx: ProvideTextTy,
{
    IterBackwardsOwned {
        inner: iter_forwards_owned(text, style),
    }
}

impl<Cx> Iterator for IterForwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    type Item = OrderedTextItem<Cx>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chars.next().map(|(index, c)| OrderedTextItem {
            c,
            style: self.style.clone(),
            index,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chars.size_hint()
    }
}

impl<Cx> DoubleEndedIterator for IterForwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.chars.next_back().map(|(index, c)| OrderedTextItem {
            c,
            style: self.style.clone(),
            index,
        })
    }
}

impl<Cx> FusedIterator for IterForwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
}

impl<Cx> Iterator for IterBackwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    type Item = OrderedTextItem<Cx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<Cx> DoubleEndedIterator for IterBackwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<Cx> FusedIterator for IterBackwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
}

impl<Cx> Iterator for IterForwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    type Item = OrderedTextItem<Cx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<Cx> DoubleEndedIterator for IterForwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<Cx> FusedIterator for IterForwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
}

impl<Cx> Iterator for IterBackwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    type Item = OrderedTextItem<Cx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<Cx> DoubleEndedIterator for IterBackwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<Cx> FusedIterator for IterBackwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
}

impl<Cx> Clone for IterForwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            chars: self.chars.clone(),
            style: self.style.clone(),
        }
    }
}

impl<Cx> Debug for IterForwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IterForwards")
            .field("chars", &self.chars)
            .field("style", &self.style)
            .finish()
    }
}

impl<Cx> Clone for IterBackwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Cx> Debug for IterBackwards<'_, Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IterBackwards")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<Cx> Clone for IterForwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _str: self._str.clone(),
        }
    }
}

impl<Cx> Debug for IterForwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IterForwardsOwned")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl<Cx> Clone for IterBackwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Cx> Debug for IterBackwardsOwned<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IterBackwardsOwned")
            .field("inner", &self.inner)
            .finish()
    }
}
