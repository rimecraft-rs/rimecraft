//! Cells that stores a type of data with a reference
//! variant or an owned variant.

#![no_std]

extern crate alloc;

use core::ops::{Deref, DerefMut};

use alloc::borrow::ToOwned;

/// Cells that stores a type of data with a reference
/// variant or an owned variant.
#[allow(clippy::exhaustive_enums)]
#[derive(Debug)]
pub enum Maybe<'a, T: ?Sized, Owned = SimpleOwned<T>> {
    /// The variant that contains a reference.
    Borrowed(&'a T),
    /// The variant that contains an owned value.
    Owned(Owned),
}

impl<T: ?Sized, Owned> Maybe<'_, T, Owned>
where
    T: ToOwned,
    <T as ToOwned>::Owned: Into<Owned>,
{
    /// Converts the cell into an owned value.
    pub fn into_owned(self) -> Owned {
        match self {
            Maybe::Borrowed(val) => val.to_owned().into(),
            Maybe::Owned(owned) => owned,
        }
    }

    /// Returns a mutable reference to the owned value, cloning it if necessary.
    pub fn get_mut(this: &mut Self) -> &mut Owned
    where
        Owned: DerefMut<Target = Owned>,
    {
        match this {
            Maybe::Borrowed(val) => {
                *this = Maybe::Owned(val.to_owned().into());
                Self::get_mut(this)
            }
            Maybe::Owned(owned) => owned,
        }
    }
}

impl<T: ?Sized, Owned> Deref for Maybe<'_, T, Owned>
where
    Owned: Deref<Target = T>,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Maybe::Borrowed(val) => val,
            Maybe::Owned(owned) => owned,
        }
    }
}

/// A cell that simply owns a value.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct SimpleOwned<T>(pub T);

impl<T> Deref for SimpleOwned<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for SimpleOwned<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}
