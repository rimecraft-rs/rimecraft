//! Tracked entity data used to being encoded and decoded for networking.

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use edcode2::{Buf, BufMut, Decode, Encode};
use local_cx::{
    ProvideLocalCxTy, WithLocalCx,
    dyn_codecs::{EdcodeCodec, UnsafeEdcodeCodec},
    edcode_codec,
};

#[doc(hidden)]
pub trait DynClone<'a>: 'a {
    fn erased_clone(&self) -> Box<dyn DynClone<'a> + Send + Sync + 'a>;
}

impl<'a, T: Clone + Send + Sync + 'a> DynClone<'a> for T {
    #[inline]
    fn erased_clone(&self) -> Box<dyn DynClone<'a> + Send + Sync + 'a> {
        Box::new(self.clone())
    }
}

impl<'a> Clone for Box<dyn DynClone<'a> + Send + Sync + 'a> {
    #[inline]
    fn clone(&self) -> Self {
        (**self).erased_clone()
    }
}

/// Entity **data accessor** that is held by entities for getting and setting data values.
#[doc(alias = "EntityDataAccessor")]
#[repr(transparent)]
pub struct TrackedData<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    raw: ErasedTrackedData<'a, Cx>,
    _marker: PhantomData<T>,
}

struct ErasedTrackedData<'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    id: u32,
    codec: UnsafeEdcodeCodec<
        Cx::LocalContext<'a>,
        dyn DynClone<'a> + Send + Sync + 'a,
        dyn DynClone<'a> + 'a,
    >,
}

impl<'a, T, Cx> TrackedData<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    /// Creates a new data accessor with the given id and codec.
    #[inline]
    pub const fn with_codec(
        id: u32,
        codec: EdcodeCodec<
            T,
            Cx::LocalContext<'a>,
            dyn DynClone<'a> + Send + Sync + 'a,
            dyn DynClone<'a> + 'a,
        >,
    ) -> Self {
        Self {
            raw: ErasedTrackedData {
                id,
                codec: codec.codec,
            },
            _marker: PhantomData,
        }
    }

    /// Gets the id of this tracked data.
    #[inline]
    pub const fn id(&self) -> u32 {
        self.raw.id
    }
}

impl<'a, T, Cx> TrackedData<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
    T: Clone
        + Send
        + Sync
        + for<'br, 'buf> Encode<WithLocalCx<&'br mut (dyn BufMut + 'buf), Cx::LocalContext<'a>>>
        + for<'br, 'buf> Decode<'static, WithLocalCx<&'br mut (dyn Buf + 'buf), Cx::LocalContext<'a>>>
        + 'a,
{
    /// Creates a new data accessor using `T`'s [`Encode`] and [`Decode`] implementations.
    pub const fn new(id: u32) -> Self {
        Self::with_codec(
            id,
            edcode_codec!(Local<Cx::LocalContext<'a>> T: DynClone + 'a),
        )
    }
}

impl<T, Cx> Hash for TrackedData<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.id.hash(state);
    }
}

impl<T, Cx> PartialEq for TrackedData<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw.id == other.raw.id
    }
}

impl<T, Cx> Eq for TrackedData<'_, T, Cx> where Cx: ProvideLocalCxTy {}

impl<Cx> Clone for ErasedTrackedData<'_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Cx> Copy for ErasedTrackedData<'_, Cx> where Cx: ProvideLocalCxTy {}

impl<T, Cx> Clone for TrackedData<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Cx> Copy for TrackedData<'_, T, Cx> where Cx: ProvideLocalCxTy {}

pub struct DataTracker {}

pub struct DataTrackerEntry<'a, T: ?Sized, Cx>
where
    Cx: ProvideLocalCxTy,
{
    dirty: bool,
    may_changed: bool,

    // erased data so we can make this type exotically sized
    data: ErasedTrackedData<'a, Cx>,

    value: T,
}

impl<'a, T, Cx> DataTrackerEntry<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    /// Creates a new entry.
    #[inline]
    pub fn new(data: TrackedData<'a, T, Cx>, value: T) -> Self {
        Self {
            dirty: false,
            may_changed: false,
            data: data.raw,
            value,
        }
    }

    /// Gets the data accessor.
    #[inline]
    pub fn data(&self) -> &TrackedData<'a, T, Cx> {
        //SAFETY: ErasedTrackedData and TrackedData are the same ABI (repr transparent).
        unsafe { *std::ptr::from_ref(&self.data).cast() }
    }
}

impl<T: ?Sized, Cx> DataTrackerEntry<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    /// Gets the inner value.
    #[inline]
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Gets the mutable inner value.
    ///
    /// *This method may comes with side-effects. Use [`Self::get`] when possible.*
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.may_changed = true;
        &mut self.value
    }

    /// Checks whether this value is dirty.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Sets the dirty flag.
    #[inline]
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// Checks whether this entry's [`Self::get_mut`] method has been called.
    #[inline]
    pub fn is_unchanged(&self) -> bool {
        !self.may_changed
    }
}

impl<T: ?Sized + Debug, Cx> Debug for DataTrackerEntry<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTrackerEntry")
            .field("data", &self.data.id)
            .field("value", &&self.value)
            .field("dirty", &self.dirty)
            .finish_non_exhaustive()
    }
}

impl<T, Cx> Debug for TrackedData<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TrackedData")
            .field(&self.raw.id)
            .finish_non_exhaustive()
    }
}
