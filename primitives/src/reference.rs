use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

/// Represents immutable reference of a value.
#[derive(Debug)]
#[repr(transparent)]
pub struct Reference<'a, T: 'a + ?Sized>(pub &'a T);

impl<'a, T: 'a + ?Sized> Copy for Reference<'a, T> {}

impl<'a, T: 'a + ?Sized> Clone for Reference<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: 'a + ?Sized> Deref for Reference<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T> From<T> for Reference<'static, T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(Box::leak(Box::new(value)))
    }
}

impl<'a, T: 'a> From<&'a T> for Reference<'a, T> {
    #[inline]
    fn from(value: &'a T) -> Self {
        Self(value)
    }
}

impl<'a, T: 'a> Eq for Reference<'a, T> where T: ?Sized {}

impl<'a, T: 'a> PartialEq for Reference<'a, T>
where
    T: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a, T: 'a> Hash for Reference<'a, T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as *const T as usize).hash(state)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct PartialEqRef<'a, T: ?Sized>(pub crate::Ref<'a, T>);

impl<'a, T: 'a + ?Sized> Copy for PartialEqRef<'a, T> {}

impl<'a, T: 'a + ?Sized> Clone for PartialEqRef<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: 'a + ?Sized> Deref for PartialEqRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0 .0
    }
}

impl<T> From<T> for PartialEqRef<'static, T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(Reference(Box::leak(Box::new(value))))
    }
}

impl<'a, T: 'a> From<&'a T> for PartialEqRef<'a, T> {
    #[inline]
    fn from(value: &'a T) -> Self {
        Self(Reference(value))
    }
}

impl<'a, T: 'a> Eq for PartialEqRef<'a, T> where T: ?Sized + PartialEq {}

impl<'a, T: 'a> PartialEq for PartialEqRef<'a, T>
where
    T: ?Sized + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || self.0 .0 == other.0 .0
    }
}

impl<'a, T: 'a> Hash for PartialEqRef<'a, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state)
    }
}
