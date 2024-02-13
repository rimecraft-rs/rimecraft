//! Attachments are a way to attach arbitrary data to
//! an object. This is useful for storing data that
//! is not directly related to the object, but is
//! still useful to store.

use std::{
    any::Any,
    borrow::Borrow,
    collections::HashMap,
    convert::Infallible,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "serde")]
pub mod serde;

/// Type that is able to attach on an
/// [`Attachments`] instance.
pub trait Attach<K>: Sized {
    /// The actual stored type.
    type Attached;

    /// The error type.
    type Error;

    /// Called before the type is attached.
    ///
    /// # Errors
    ///
    /// Returns an error if the attachment failed.
    #[inline]
    #[allow(unused_variables)]
    fn attach(&mut self, attachments: &mut Attachments<K>, key: &K) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Converts the type into the attached type.
    fn into_attached(self) -> Self::Attached;
}

/// Type of an attachment.
#[derive(Debug)]
pub struct Type<K, T> {
    key: K,
    _marker: PhantomData<T>,
}

pub use Type as AttachmentType;

impl<K, T> Type<K, T> {
    /// Creates a new [`Type`] from given key.
    #[inline]
    pub const fn new(key: K) -> Self {
        Self {
            key,
            _marker: PhantomData,
        }
    }

    /// Returns the key of the type.
    #[inline]
    pub const fn key(&self) -> &K {
        &self.key
    }
}

type RawAttachments<K> = HashMap<K, Box<dyn Any + Send + Sync>>;

/// Manager of attachments.
#[derive(Debug)]
pub struct Attachments<K> {
    raw: RawAttachments<K>,

    #[cfg(feature = "serde")]
    serde_state: crate::serde::State<K>,
}

impl<K> Attachments<K> {
    /// Creates a new [`Attachments`] instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            raw: HashMap::new(),
            #[cfg(feature = "serde")]
            serde_state: Default::default(),
        }
    }
}

impl<K> Default for Attachments<K> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq> Attachments<K> {
    /// Attaches a value to the attachments with
    /// given [`Type`].
    ///
    /// # Errors
    ///
    /// Returns an error if the attachment failed.\
    /// See [`Attach::attach`].
    #[inline]
    pub fn attach<T: Attach<K>, Q>(
        &mut self,
        ty: &Type<Q, <T as Attach<K>>::Attached>,
        mut val: T,
    ) -> Result<(), <T as Attach<K>>::Error>
    where
        <T as Attach<K>>::Attached: Any + Send + Sync + 'static,
        Q: ToOwned,
        K: From<<Q as ToOwned>::Owned>,
    {
        let key = ty.key.to_owned().into();
        val.attach(self, &key)?;
        self.raw.insert(key, Box::new(val.into_attached()));
        Ok(())
    }

    /// Returns a reference to the attached value
    /// with given [`Type`].
    #[inline]
    pub fn get<'a, T, Q>(
        &'a self,
        ty: &Type<&Q, <T as Attach<K>>::Attached>,
    ) -> Option<<<T as Attach<K>>::Attached as AsAttachment<'a>>::Output>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        T: Attach<K>,
        <T as Attach<K>>::Attached: AsAttachment<'a> + Any + Send + Sync + 'static,
    {
        self.raw
            .get(ty.key)
            .and_then(|val| val.downcast_ref::<<T as Attach<K>>::Attached>())
            .map(|val| val.as_attachment())
    }

    /// Returns a mutable reference to the attached
    /// value with given [`Type`].
    #[inline]
    pub fn get_mut<'a, T, Q>(
        &'a mut self,
        ty: &Type<&Q, T>,
    ) -> Option<<<T as Attach<K>>::Attached as AsAttachmentMut<'a>>::Output>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        T: Attach<K>,
        <T as Attach<K>>::Attached: AsAttachmentMut<'a> + Any + Send + Sync + 'static,
    {
        self.raw
            .get_mut(ty.key)
            .and_then(|val| val.downcast_mut::<<T as Attach<K>>::Attached>())
            .map(|val| val.as_attachment_mut())
    }

    /// Whether the attachments contains a value
    /// with given [`Type`].
    #[inline]
    pub fn contains<T: Any, Q>(&self, ty: &Type<&Q, T>) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        T: Send + Sync + 'static,
    {
        self.raw.get(ty.key).is_some_and(|val| val.is::<T>())
    }
}

impl<K> Attachments<K> {
    /// Whether the persistent data queue is empty.
    #[cfg(feature = "serde")]
    #[inline]
    pub fn is_persistent_data_empty(&self) -> bool {
        self.serde_state.ser.is_empty() && self.serde_state.update.is_empty()
    }
}

impl<K, T> Clone for Type<K, T>
where
    K: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: PhantomData,
        }
    }
}

impl<K, T> Copy for Type<K, T> where K: Copy {}

impl<K, T> PartialEq for Type<K, T>
where
    K: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K, T> Eq for Type<K, T> where K: Eq {}

impl<K, T> From<K> for Type<K, T> {
    #[inline]
    fn from(key: K) -> Self {
        Self::new(key)
    }
}

#[allow(dead_code)]
type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// Trait for types that can be dereferenced
/// into an attachment.
pub trait AsAttachment<'a> {
    /// The target type.
    type Target: ?Sized + 'a;

    /// The output type.
    type Output: Deref<Target = Self::Target> + 'a;

    /// Dereferences the type into an attachment.
    fn as_attachment(&'a self) -> Self::Output;
}

/// Trait for types that can be dereferenced
/// mutably into an attachment.
pub trait AsAttachmentMut<'a>: AsAttachment<'a> {
    /// The output type.
    type Output: Deref<Target = <Self as AsAttachment<'a>>::Target> + DerefMut + 'a;

    /// Dereferences the type mutably into an attachment.
    fn as_attachment_mut(&'a mut self) -> <Self as AsAttachmentMut<'a>>::Output;
}

impl<'a, T: ?Sized> AsAttachment<'a> for T
where
    T: Deref + 'a,
    <T as Deref>::Target: 'a,
{
    type Target = <T as Deref>::Target;
    type Output = &'a Self::Target;

    #[inline]
    fn as_attachment(&'a self) -> Self::Output {
        self
    }
}

impl<'a, T: ?Sized> AsAttachmentMut<'a> for T
where
    T: DerefMut + 'a,
    <T as Deref>::Target: 'a,
{
    type Output = &'a mut <Self as AsAttachment<'a>>::Target;

    #[inline]
    fn as_attachment_mut(&'a mut self) -> <Self as AsAttachmentMut<'a>>::Output {
        &mut *self
    }
}

/// A simple attachment that does not require
/// any attachment logic.
#[derive(Debug)]
pub struct Simple<T>(pub T);

impl<T, K> Attach<K> for Simple<T> {
    type Attached = Self;
    type Error = Infallible;

    #[inline]
    fn into_attached(self) -> Self::Attached {
        self
    }
}

impl<T> Deref for Simple<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Simple<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests;
