use std::{fmt::Debug, iter::FusedIterator};

use remap::remap;

use crate::{
    ProvideTextTy, Style,
    ordered::{OrderedText, OrderedTextItem},
};

/// `OrderedText` returned from [`from_iter`].
#[derive(Debug, Clone)]
pub struct FromIter<I>(I);

impl<I, Cx> OrderedText<Cx> for FromIter<I>
where
    Cx: ProvideTextTy,
    I: IntoIterator<Item = OrderedTextItem<Cx>, IntoIter: FusedIterator> + Clone,
{
    type Iter<'a>
        = I::IntoIter
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.0.clone().into_iter()
    }
}

/// Creates an [`OrderedText`] from a cloneable iterator.
///
/// This could be used to create almost all types of `OrderedText`.
#[inline]
#[remap(yarn = "of")]
#[doc(alias = "composite")] // mojmaps name but repeated elsewhere
pub fn from_iter<I>(iter: I) -> FromIter<I> {
    FromIter(iter)
}

/// `OrderedText` returned from [`empty`].
#[derive(Debug, Clone, Copy)]
pub struct Empty();

impl<Cx> OrderedText<Cx> for Empty
where
    Cx: ProvideTextTy,
{
    type Iter<'a>
        = std::iter::Empty<OrderedTextItem<Cx>>
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        std::iter::empty()
    }
}

/// Creates an empty [`OrderedText`].
#[inline]
#[remap(yarn = "empty")]
#[doc(alias = "composite")]
pub fn empty() -> Empty {
    Empty()
}

/// `OrderedText` returned from [`OrderedText::map`].
#[derive(Debug, Clone)]
pub struct Map<T, F>(pub(super) T, pub(super) F);

impl<T, F, Cx> OrderedText<Cx> for Map<T, F>
where
    T: OrderedText<Cx>,
    F: Fn(OrderedTextItem<Cx>) -> OrderedTextItem<Cx>,
    Cx: ProvideTextTy,
{
    type Iter<'a>
        = std::iter::Map<T::Iter<'a>, &'a F>
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().map(&self.1)
    }

    #[inline]
    fn peek_iter<P, U>(&self, p: P) -> U
    where
        P: FnOnce(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>) -> U,
    {
        self.0.peek_iter(|iter| p(&mut iter.map(&self.1)))
    }
}

/// `OrderedText` returned from [`OrderedText::chain`].
#[derive(Debug, Clone)]
pub struct Chain<T1, T2>(pub(super) T1, pub(super) T2);

impl<T1, T2, Cx> OrderedText<Cx> for Chain<T1, T2>
where
    T1: OrderedText<Cx>,
    T2: OrderedText<Cx>,
    Cx: ProvideTextTy,
{
    type Iter<'a>
        = std::iter::Chain<T1::Iter<'a>, T2::Iter<'a>>
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().chain(self.1.iter())
    }
}

/// `OrderedText` returned from [`OrderedText::rev`].
#[derive(Debug, Clone)]
pub struct Rev<T>(pub(super) T);

impl<T, Cx> OrderedText<Cx> for Rev<T>
where
    T: OrderedText<Cx>,
    for<'a> T::Iter<'a>: DoubleEndedIterator,
    Cx: ProvideTextTy,
{
    type Iter<'a>
        = std::iter::Rev<T::Iter<'a>>
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().rev()
    }
}

/// `OrderedText` returned from [`once`].
pub struct Once<Cx>
where
    Cx: ProvideTextTy,
{
    c: char,
    style: Style<Cx::StyleExt>,
}

impl<Cx> OrderedText<Cx> for Once<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    type Iter<'a>
        = std::iter::Once<OrderedTextItem<Cx>>
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        std::iter::once(OrderedTextItem {
            index: 0,
            c: self.c,
            style: self.style.clone(),
        })
    }
}

impl<Cx> Debug for Once<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Once")
            .field("c", &self.c)
            .field("style", &self.style)
            .finish()
    }
}

impl<Cx> Clone for Once<Cx>
where
    Cx: ProvideTextTy,
    Cx::StyleExt: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            c: self.c,
            style: self.style.clone(),
        }
    }
}

/// Creates an [`OrderedText`] with a single character.
#[inline]
#[remap(yarn = "styled", mojmaps = "codepoint")]
pub fn once<Cx>(c: char, style: Style<Cx::StyleExt>) -> Once<Cx>
where
    Cx: ProvideTextTy,
{
    Once { c, style }
}
