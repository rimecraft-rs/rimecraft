use std::{
    any::Any,
    borrow::Borrow,
    collections::HashMap,
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
    type Attached: From<Self>;

    /// The error type.
    type Error;

    /// Called before the type is attached.
    #[inline]
    #[allow(unused_variables)]
    fn attach(&mut self, attachments: &mut Attachments<K>, key: &K) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Type of an attachment.
#[derive(Debug)]
pub struct Type<K, T> {
    key: K,
    _marker: PhantomData<T>,
}

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
pub struct Attachments<K> {
    raw: RawAttachments<K>,

    #[cfg(feature = "serde")]
    serde_state: crate::serde::State<K>,
}

impl<K: 'static> Attachments<K> {
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

impl<K: 'static> Default for Attachments<K> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq> Attachments<K> {
    /// Attaches a value to the attachments with
    /// given [`Type`].
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
        self.raw
            .insert(key, Box::new(<T as Attach<K>>::Attached::from(val)));
        Ok(())
    }

    /// Returns a reference to the attached value
    /// with given [`Type`].
    #[inline]
    pub fn get<'a, T: Any, Q>(
        &'a self,
        ty: &Type<Q, T>,
    ) -> Option<<<T as Attach<K>>::Attached as AsAttachment<'a>>::Output>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
        T: Attach<K>,
        <T as Attach<K>>::Attached: AsAttachment<'a> + Any + Send + Sync + 'static,
    {
        self.raw
            .get(&ty.key)
            .and_then(|val| val.downcast_ref::<<T as Attach<K>>::Attached>())
            .map(|val| val.as_attachment())
    }

    /// Returns a mutable reference to the attached
    /// value with given [`Type`].
    #[inline]
    pub fn get_mut<'a, T: Any, Q>(
        &'a mut self,
        ty: &Type<Q, T>,
    ) -> Option<<<T as Attach<K>>::Attached as AsAttachmentMut<'a>>::Output>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
        T: Attach<K>,
        <T as Attach<K>>::Attached: AsAttachmentMut<'a> + Any + Send + Sync + 'static,
    {
        self.raw
            .get_mut(&ty.key)
            .and_then(|val| val.downcast_mut::<<T as Attach<K>>::Attached>())
            .map(|val| val.as_attachment_mut())
    }

    /// Whether the attachments contains a value
    /// with given [`Type`].
    #[inline]
    pub fn contains<T: Any, Q>(&self, ty: &Type<Q, T>) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
        T: Send + Sync + 'static,
    {
        self.raw.get(&ty.key).is_some_and(|val| val.is::<T>())
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

pub trait AsAttachment<'a> {
    type Target: ?Sized + 'a;

    type Output: Deref<Target = Self::Target> + 'a;

    fn as_attachment(&'a self) -> Self::Output;
}

pub trait AsAttachmentMut<'a>: AsAttachment<'a> {
    type Output: Deref<Target = <Self as AsAttachment<'a>>::Target> + DerefMut + 'a;

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
