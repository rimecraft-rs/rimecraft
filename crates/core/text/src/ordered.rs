//! Represents an ordered character sequence with associated [`Style`]s.

use std::{
    fmt::Debug,
    iter::FusedIterator,
    mem::{ManuallyDrop, MaybeUninit},
    rc::Rc,
    sync::Arc,
};

use remap::{remap, remap_method};
use smallbox::SmallBox;

use crate::{ProvideTextTy, Style};

mod func;
mod iter;

pub use func::*;
pub use iter::*;

/// An item in an iterator over text items with associated [`Style`]s.
///
/// See: [`OrderedText`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderedTextItem<Cx>
where
    Cx: ProvideTextTy,
{
    /// The character.
    pub c: char,
    /// The [`Style`] associated with the character.
    pub style: Style<Cx::StyleExt>,

    /// The index of the character in the original utf-8 string.
    pub index: usize,
}

/// An object that can supply [`OrderedTextItem`]s to a visitor.
#[remap(yarn = "OrderedText", mojmaps = "FormattedCharSequence")]
pub trait OrderedText<Cx>
where
    Cx: ProvideTextTy,
{
    /// The iterator type over the characters of this text with associated [`Style`]s.
    type Iter<'a>: FusedIterator<Item = OrderedTextItem<Cx>>
    where
        Self: 'a;

    /// Returns an iterator over the characters of this text with associated [`Style`]s.
    fn iter(&self) -> Self::Iter<'_>;

    /// Accepts a peeker function and returns the result of the first peeker call if `Some` is returned.
    ///
    /// This should only be used when targeting an [`ErasedOrderedText`], or use `iter` instead.
    #[doc(hidden)]
    #[inline]
    fn peek_iter<P, U>(&self, p: P) -> U
    where
        P: FnOnce(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>) -> U,
    {
        p(&mut self.iter())
    }

    /// Accepts a visitor function and returns the result of the first visitor call if `Some` is returned.
    #[deprecated = "use `iter` instead"]
    #[remap_method(yarn = "accept", mojmaps = "accept")]
    fn accept<V, U>(&self, mut visitor: V) -> Option<U>
    where
        V: FnMut(OrderedTextItem<Cx>) -> Option<U>,
    {
        for item in self.iter() {
            if let Some(result) = visitor(item) {
                return Some(result);
            }
        }
        None
    }

    // utilities

    /// Maps the items of this text into another item.
    #[inline]
    #[remap_method(yarn = "map", mojmaps = "decorateOutput")]
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        F: Fn(OrderedTextItem<Cx>) -> OrderedTextItem<Cx>,
        Self: Sized,
    {
        Map(self, f)
    }

    /// Chains two ordered texts together.
    #[inline]
    #[remap_method(yarn = "concat", mojmaps = "composite")]
    #[remap_method(yarn = "innerConcat", mojmaps = "fromPair")]
    fn chain<T>(self, other: T) -> Chain<Self, T>
    where
        T: OrderedText<Cx>,
        Self: Sized,
    {
        Chain(self, other)
    }

    /// Reverses the order of the items in this text.
    #[inline]
    #[doc(alias = "reverse")]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized,
    {
        Rev(self)
    }
}

/// Erased variant of [`OrderedText`].
pub trait ErasedOrderedText<Cx>: sealed::Erased<Cx>
where
    Cx: ProvideTextTy,
{
}

/// The iterator type for [`ErasedOrderedText`].
pub struct ErasedIter<'a, Cx>(
    SmallBox<dyn FusedIterator<Item = OrderedTextItem<Cx>> + 'a, smallbox::space::S4>,
)
where
    Cx: ProvideTextTy;

impl<Cx> OrderedText<Cx> for dyn ErasedOrderedText<Cx> + '_
where
    Cx: ProvideTextTy,
{
    type Iter<'a>
        = ErasedIter<'a, Cx>
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        ErasedIter(self.erased_iter())
    }

    fn peek_iter<P, U>(&self, p: P) -> U
    where
        P: FnOnce(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>) -> U,
    {
        let mut val = MaybeUninit::uninit();
        let mut p = ManuallyDrop::new(p);
        self.erased_peek_iter(&mut |iter| {
            // SAFETY: implementation of `sealed::Erased` guarantees the peeker is called only once
            let result = unsafe { ManuallyDrop::take(&mut p) }(iter);
            val.write(result);
        });
        // SAFETY: implementation of `sealed::Erased` guarantees the peeker is called
        unsafe { val.assume_init() }
    }
}

mod sealed {
    use smallbox::SmallBox;

    use super::*;

    pub trait Erased<Cx>
    where
        Cx: ProvideTextTy,
    {
        fn erased_peek_iter(
            &self,
            f: &mut dyn FnMut(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>),
        );

        fn erased_iter(
            &self,
        ) -> SmallBox<dyn FusedIterator<Item = OrderedTextItem<Cx>> + '_, smallbox::space::S4>;
    }

    impl<Cx, T: ?Sized> Erased<Cx> for T
    where
        Cx: ProvideTextTy,
        T: OrderedText<Cx>,
    {
        #[inline]
        fn erased_peek_iter(
            &self,
            f: &mut dyn FnMut(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>),
        ) {
            f(&mut self.iter())
        }

        #[inline]
        fn erased_iter(
            &self,
        ) -> SmallBox<dyn FusedIterator<Item = OrderedTextItem<Cx>> + '_, smallbox::space::S4>
        {
            smallbox::smallbox!(self.iter())
        }
    }
}

impl<Cx> Iterator for ErasedIter<'_, Cx>
where
    Cx: ProvideTextTy,
{
    type Item = OrderedTextItem<Cx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<Cx> FusedIterator for ErasedIter<'_, Cx> where Cx: ProvideTextTy {}

impl<Cx, T: ?Sized> ErasedOrderedText<Cx> for T
where
    Cx: ProvideTextTy,
    T: OrderedText<Cx>,
{
}

macro_rules! impl_ordered_text_for_dyn {
    ($($t:path),*$(,)?) => {
        impl<Cx> OrderedText<Cx> for dyn ErasedOrderedText<Cx> + $($t +)* '_
        where
            Cx: ProvideTextTy,
        {
            type Iter<'a>
                = ErasedIter<'a, Cx>
            where
                Self: 'a;

            #[inline]
            fn iter(&self) -> Self::Iter<'_> {
                <dyn ErasedOrderedText<Cx> + '_>::iter(self)
            }

            #[inline]
            fn peek_iter<P, U>(&self, p: P) -> U
            where
                P: FnOnce(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>) -> U,
            {
                <dyn ErasedOrderedText<Cx> + '_>::peek_iter(self, p)
            }
        }
    };
}

impl_ordered_text_for_dyn!(Send);
impl_ordered_text_for_dyn!(Sync);
impl_ordered_text_for_dyn!(Send, Sync);
impl_ordered_text_for_dyn!(Unpin);
impl_ordered_text_for_dyn!(Send, Unpin);
impl_ordered_text_for_dyn!(Sync, Unpin);
impl_ordered_text_for_dyn!(Send, Sync, Unpin);

impl<Cx> Debug for ErasedIter<'_, Cx>
where
    Cx: ProvideTextTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasedIter").finish_non_exhaustive()
    }
}

impl<'r, Cx, T> OrderedText<Cx> for &'r T
where
    Cx: ProvideTextTy,
    T: OrderedText<Cx> + ?Sized,
{
    type Iter<'a>
        = T::Iter<'r>
    where
        Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        (**self).iter()
    }

    #[inline]
    fn peek_iter<P, U>(&self, p: P) -> U
    where
        P: FnOnce(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>) -> U,
    {
        (**self).peek_iter(p)
    }
}

macro_rules! impl_ordered_text_for_smart_ptr {
    ($($t:ident),*$(,)?) => {
        $(
        impl<Cx, T> OrderedText<Cx> for $t<T>
        where
            Cx: ProvideTextTy,
            T: OrderedText<Cx> + ?Sized,
        {
            type Iter<'a>
                = T::Iter<'a>
            where
                Self: 'a;

            #[inline]
            fn iter(&self) -> Self::Iter<'_> {
                (**self).iter()
            }

            #[inline]
            fn peek_iter<P, U>(&self, p: P) -> U
            where
                P: FnOnce(&mut dyn FusedIterator<Item = OrderedTextItem<Cx>>) -> U,
            {
                (**self).peek_iter(p)
            }
        }
        )*
    };
}

impl_ordered_text_for_smart_ptr!(Box, Arc, Rc);
