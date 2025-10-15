//! Tracked entity data used to being encoded and decoded for networking.

use std::{any::TypeId, fmt::Debug, hash::Hash, marker::PhantomData, sync::OnceLock};

use ahash::AHashMap;
use edcode2::{Buf, BufExt as _, BufMut, BufMutExt as _, Decode, Encode};
use global_cx::GlobalContext;
use local_cx::{
    ForwardToWithLocalCx, LocalContext, ProvideLocalCxTy,
    dyn_codecs::{EdcodeCodec, UnsafeEdcodeCodec},
};
use marking::LeakedPtrMarker;
use maybe::Maybe;
use parking_lot::Mutex;
use registry::Reg;

use crate::{EntityCx, EntityType};

mod iter;

pub use iter::*;

/// The maximum number of data values.
pub const MAX_DATA_VALUES: usize = u8::MAX as usize;

/// The maximum data ID.
pub const MAX_DATA_ID: u32 = MAX_DATA_VALUES as u32 - 1;

static ENTITY_TY_DATA_IDS: OnceLock<Mutex<AHashMap<(TypeId, LeakedPtrMarker), u32>>> =
    OnceLock::new();

/// Global context types satisfying use of entity data.
pub trait EntityDataCx<'a>: ProvideLocalCxTy + GlobalContext {}

impl<T> EntityDataCx<'_> for T where T: ProvideLocalCxTy + GlobalContext {}

/// A trait implemented by objects whose inner data is tracked by [`DataTracker`].
pub trait DataTracked<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    /// Called on the client when the tracked data is set.
    #[inline]
    fn on_tracked_data_set(&mut self, tracked: &RawTrackedData<'a, Cx>) {
        let _ = tracked;
    }
}

/// A registry of [`Codec`]s.
pub struct CodecRegistry<'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    codecs: Vec<UnsafeCodec<'a, Cx>>,
}

/// Unique ID of a [`Codec`] registered in a [`CodecRegistry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CodecId(u32);

impl<B> Encode<B> for CodecId
where
    B: BufMut,
{
    #[inline]
    fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
        buf.put_variable(self.0);
        Ok(())
    }
}

impl<'de, B> Decode<'de, B> for CodecId
where
    B: Buf,
{
    #[inline]
    fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
        Ok(CodecId(buf.get_variable::<u32>()))
    }
}

impl<'a, Cx> CodecRegistry<'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    /// Creates a new empty registry.
    #[inline]
    pub fn new() -> Self {
        Self { codecs: Vec::new() }
    }

    /// Registers a new codec and returns its ID.
    #[inline]
    pub fn register<T>(&mut self, codec: Codec<'a, T, Cx>) -> CodecId {
        self.codecs.push(codec.codec);
        CodecId(self.codecs.len() as u32 - 1)
    }

    /// Gets a codec by its ID.
    #[inline]
    pub fn get_raw(&self, id: CodecId) -> Option<UnsafeCodec<'a, Cx>> {
        self.codecs.get(id.0 as usize).copied()
    }
}

impl<Cx> Default for CodecRegistry<'_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Cx> Debug for CodecRegistry<'_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodecRegistry")
            .field("codecs", &self.codecs)
            .finish()
    }
}

/// A protected `u32` type representing ID of a [`TrackedData`].
///
/// See [`register_data`] for obtaining one.
#[derive(Debug)]
pub struct DataId(u32, TypeId);

/// Registers a [`TrackedData`] by returning its unique ID.
///
/// # Panics
///
/// Panics if count of data IDs exceeds [`MAX_DATA_ID`].
pub fn register_data<'a, Cx: EntityCx<'a>>(ty: EntityType<'a, Cx>) -> DataId {
    let typeid = ty.erased_typeid();
    let marker = Reg::to_entry(ty).marker_leaked();

    let mut map = ENTITY_TY_DATA_IDS
        .get_or_init(|| Mutex::new(AHashMap::new()))
        .lock();

    DataId(
        match map.entry((typeid, marker)) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                let i = e.get_mut();
                assert!(
                    *i < MAX_DATA_ID,
                    "count of data IDs exceeded for this entity type. expected: <= {}, actual: {}",
                    MAX_DATA_ID,
                    *i + 1
                );
                *i += 1;
                *i
            }
            std::collections::hash_map::Entry::Vacant(e) => *e.insert(0),
        },
        typeid,
    )
}

#[doc(hidden)]
pub trait Value<'a, Cx: 'a>: 'a {
    fn erased_clone(&self) -> Box<DynValue<'a, Cx>>;
    fn erased_typeid(&self) -> TypeId;

    unsafe fn swap_entry(&mut self, entry: &mut DataTrackerEntry<'a, DynValue<'a, Cx>, Cx>)
    where
        Cx: EntityDataCx<'a>;
}

type DynValue<'a, Cx> = dyn Value<'a, Cx> + Send + Sync + 'a;

impl<'a, T: Clone + Send + Sync + 'a, Cx: 'a> Value<'a, Cx> for T {
    #[inline]
    fn erased_clone(&self) -> Box<DynValue<'a, Cx>> {
        Box::new(self.clone())
    }

    #[inline]
    fn erased_typeid(&self) -> TypeId {
        typeid::of::<T>()
    }

    unsafe fn swap_entry(&mut self, entry: &mut DataTrackerEntry<'a, DynValue<'a, Cx>, Cx>)
    where
        Cx: EntityDataCx<'a>,
    {
        assert_eq!(
            typeid::of::<T>(),
            entry.value.erased_typeid(),
            "type mismatch"
        );

        //SAFETY: we have the type checked but the lifetime is unsure. so thats why this function is unsafe.
        let entry =
            unsafe { &mut *std::ptr::from_mut(entry).cast::<DataTrackerEntry<'a, T, Cx>>() };

        std::mem::swap(self, &mut entry.value);
    }
}

impl<'a, Cx: 'a> Clone for Box<DynValue<'a, Cx>> {
    #[inline]
    fn clone(&self) -> Self {
        (**self).erased_clone()
    }
}

impl<'a, Cx: 'a> ToOwned for DynValue<'a, Cx> {
    type Owned = Box<Self>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.erased_clone()
    }
}

type UnsafeCodec<'a, Cx> = UnsafeEdcodeCodec<
    <Cx as ProvideLocalCxTy>::LocalContext<'a>,
    DynValue<'a, Cx>,
    dyn Value<'a, Cx> + 'a,
>;

/// A codec used to encode and decode tracked data.
#[doc(alias = "TrackedDataHandler")]
pub type Codec<'a, T, Cx> = EdcodeCodec<
    T,
    <Cx as ProvideLocalCxTy>::LocalContext<'a>,
    DynValue<'a, Cx>,
    dyn Value<'a, Cx> + 'a,
>;

/// Entity **data accessor** that is held by entities for getting and setting data values.
#[doc(alias = "EntityDataAccessor")]
#[repr(transparent)]
pub struct TrackedData<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
{
    raw: RawTrackedData<'a, Cx>,

    // make T invariant
    _marker: PhantomData<fn(T) -> T>,
}

/// Type-erased raw representation of a [`TrackedData`].
pub struct RawTrackedData<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    id: u32,
    entity_ty_typeid: TypeId,
    codec: UnsafeCodec<'a, Cx>,
    codec_id: CodecId,
}

impl<'a, T, Cx> TrackedData<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
{
    /// Creates a new data accessor with the given id and codec.
    #[inline]
    pub const fn with_codec(id: DataId, codec: Codec<'a, T, Cx>, codec_id: CodecId) -> Self {
        Self {
            raw: RawTrackedData {
                id: id.0,
                entity_ty_typeid: id.1,
                codec: codec.codec,
                codec_id,
            },
            _marker: PhantomData,
        }
    }

    /// Gets the id of this tracked data.
    #[inline]
    pub const fn id(&self) -> u32 {
        self.raw.id()
    }
}

impl<'a, Cx> RawTrackedData<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    /// Gets the id of this tracked data.
    #[inline]
    pub const fn id(&self) -> u32 {
        self.id
    }
}

impl<'a, Cx> Hash for RawTrackedData<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.entity_ty_typeid.hash(state);
    }
}

impl<'a, T, Cx> Hash for TrackedData<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<'a, Cx> PartialEq for RawTrackedData<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.entity_ty_typeid == other.entity_ty_typeid
    }
}

impl<'a, T, Cx> PartialEq for TrackedData<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<'a, Cx> Eq for RawTrackedData<'a, Cx> where Cx: EntityDataCx<'a> {}

impl<'a, T, Cx> Eq for TrackedData<'a, T, Cx> where Cx: EntityDataCx<'a> {}

impl<'a, Cx> Clone for RawTrackedData<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Cx> Copy for RawTrackedData<'a, Cx> where Cx: EntityDataCx<'a> {}

impl<'a, T, Cx> Clone for TrackedData<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T, Cx> Copy for TrackedData<'a, T, Cx> where Cx: EntityDataCx<'a> {}

type ErasedEntry<'a, Cx> = Box<DataTrackerEntry<'a, DynValue<'a, Cx>, Cx>>;

/// Holder of an entity's all [`TrackedData`]s and their values.
pub struct DataTracker<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    entries: Box<[ErasedEntry<'a, Cx>]>,
    entity_ty_typeid: TypeId,

    dirty: bool,
}

impl<'a, Cx> DataTracker<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    fn entry<T>(&self, key: &TrackedData<'a, T, Cx>) -> Option<&DataTrackerEntry<'a, T, Cx>> {
        assert_eq!(
            key.raw.entity_ty_typeid, self.entity_ty_typeid,
            "entity class of given tracked data should match with tracker"
        );

        self.entries.get(key.raw.id as usize).map(|entry| unsafe {
            //SAFETY: `T` in `key` is invariant and here we have a registry to guarantee that the type and lifetime is the same.
            &*std::ptr::from_ref(&**entry).cast()
        })
    }

    fn entry_mut<T>(
        &mut self,
        key: &TrackedData<'a, T, Cx>,
        set_dirty: bool,
    ) -> Option<&mut DataTrackerEntry<'a, T, Cx>> {
        assert_eq!(
            key.raw.entity_ty_typeid, self.entity_ty_typeid,
            "entity class of given tracked data should match with tracker"
        );

        let entry: Option<&mut DataTrackerEntry<'a, T, Cx>> = self
            .entries
            .get_mut(key.raw.id as usize)
            .map(|entry| unsafe {
                //SAFETY: `T` in `key` is invariant and here we have a registry to guarantee that the type and lifetime is the same.
                &mut *std::ptr::from_mut(&mut **entry).cast()
            });

        if set_dirty {
            #[allow(clippy::manual_inspect)] // takes mutable reference
            entry.map(|e| {
                e.set_dirty(true);
                self.dirty = true;
                e
            })
        } else {
            entry
        }
    }

    /// Gets an immutable reference to the value of the given key.
    #[inline]
    pub fn get<'e, T: 'e>(&'e self, key: &TrackedData<'a, T, Cx>) -> Option<&'e T> {
        self.entry(key).map(DataTrackerEntry::get)
    }

    /// Gets a mutable reference to the value of the given key.
    ///
    /// *This method may comes with side-effects. Use [`Self::get`] when possible.*
    #[inline]
    pub fn get_mut<'e, T: 'e>(&'e mut self, key: &TrackedData<'a, T, Cx>) -> Option<&'e mut T> {
        self.entry_mut(key, true).map(DataTrackerEntry::get_mut)
    }

    /// Updates this tracker by the given entries which are updated entries of this tracker.
    ///
    /// This method won't tell the entity to update itself. Instead, you should provide a callback `on_update`
    /// to make the entity's data update itself. Use [`crate::RawEntity::update_tracker_entries`] if possible.
    ///
    /// # Panics
    ///
    /// - Panics if the codec of the serialized entry does not match with the entry.
    /// - Panics if type mismatch occurs.
    #[doc(alias = "write_updated_entries")]
    pub fn update_entries<'borrow, I, F>(&mut self, entries: I, mut on_update: F)
    where
        I: IntoIterator<Item = SerializedEntry<'borrow, 'a, Cx>>,
        F: FnMut(RawTrackedData<'a, Cx>),
        'a: 'borrow,
    {
        for serialized in entries {
            let Some(entry) = self.entries.get_mut(serialized.id as usize) else {
                continue;
            };
            assert_eq!(
                entry.data.codec_id, serialized.codec_id,
                "codec of serialized entry should match with entry"
            );
            let data = entry.data;

            //SAFETY: the data type at least outlives `'a`-lifetime.
            unsafe { serialized.value.into_owned().swap_entry(entry) };
            let _ = entry;

            on_update(data);
        }
    }

    /// Gets an iterator over all changed entries of this tracker.
    #[inline]
    pub fn changed_entries(&self) -> ChangedEntries<'_, 'a, Cx> {
        ChangedEntries {
            inner_iter: self.entries.iter(),
        }
    }

    /// Gets an iterator over all dirty entries of this tracker.
    ///
    /// This doesn't follow vanilla behavior as it won't clear the dirty flags.
    /// To make it clear flags automatically, see [`Self::self_cleaning_dirty_entries`].
    #[inline]
    pub fn dirty_entries(&self) -> DirtyEntries<'_, 'a, Cx> {
        DirtyEntries {
            inner_iter: self.entries.iter(),
        }
    }

    /// Gets an iterator over all dirty entries of this tracker that _automatically clears the dirty flag._
    #[inline]
    pub fn self_cleaning_dirty_entries(&mut self) -> SelfCleaningDirtyEntries<'_, 'a, Cx> {
        SelfCleaningDirtyEntries {
            inner_iter: self.entries.iter_mut(),
        }
    }
}

/// Builder of a [`DataTracker`].
pub struct DataTrackerBuilder<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    entries: Box<[Option<ErasedEntry<'a, Cx>>]>,
    entity_ty_typeid: TypeId,
    len: usize,
}

impl<'a, Cx> DataTrackerBuilder<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    /// Creates a new data tracker builder from given entity type.
    pub fn new(ty: EntityType<'a, Cx>) -> Self {
        let typeid = ty.erased_typeid();
        let marker = Reg::to_entry(ty).marker_leaked();

        let len = ENTITY_TY_DATA_IDS
            .get()
            .and_then(|map| map.lock().get(&(typeid, marker)).copied())
            .map(|id| id as usize + 1)
            .unwrap_or_default();
        let mut vec = Vec::with_capacity(len);
        vec.resize_with(len, || None);

        Self {
            entries: vec.into_boxed_slice(),
            entity_ty_typeid: typeid,
            len: 0,
        }
    }
}

impl<'a, Cx> DataTrackerBuilder<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    /// Inserts a value into this tracker if absent, or returns the given value.
    ///
    /// # Panics
    ///
    /// Panics if the entity type of given tracked data doesn't match the type of this tracker.
    pub fn insert<T>(&mut self, data: TrackedData<'a, T, Cx>, value: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'a,
    {
        assert_eq!(
            data.raw.entity_ty_typeid, self.entity_ty_typeid,
            "entity type of given tracked data should match this tracker"
        );

        if let Some(val) = self.entries.get_mut(data.id() as usize)
            && val.is_none()
        {
            *val = Some(Box::new(DataTrackerEntry::new(data, value)));
            self.len += 1;
            None
        } else {
            Some(value)
        }
    }

    /// Returns the length of this tracker.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether this tracker is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<'a, Cx> DataTrackerBuilder<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    /// Builds a data tracker.
    ///
    /// # Panics
    ///
    /// Panics if there are undefined entries in this builder (left as `None`).
    pub fn build(self) -> DataTracker<'a, Cx> {
        DataTracker {
            entries: self
                .entries
                .into_iter()
                .map(|opt| opt.expect("data value should be defined"))
                .collect(),
            entity_ty_typeid: self.entity_ty_typeid,
            dirty: false,
        }
    }
}

/// Entry of a [`DataTracker`], containing a [`TrackedData`] and its corresponding value.
pub struct DataTrackerEntry<'a, T: ?Sized, Cx>
where
    Cx: EntityDataCx<'a>,
{
    dirty: bool,
    may_changed: bool,

    // raw data so we can make this type exotically sized
    data: RawTrackedData<'a, Cx>,

    value: T,
}

impl<'a, T, Cx> DataTrackerEntry<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
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
        //SAFETY: ErasedTrackedData and TrackedData share the same ABI (repr transparent).
        unsafe { *std::ptr::from_ref(&self.data).cast() }
    }
}

impl<'a, T: ?Sized, Cx> DataTrackerEntry<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
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
        self.__get_mut_unaffected()
    }

    #[inline]
    fn __get_mut_unaffected(&mut self) -> &mut T {
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

impl<'a, Cx> DataTrackerEntry<'a, DynValue<'a, Cx>, Cx>
where
    Cx: EntityDataCx<'a>,
{
    /// Returns a serialized form of this entry for encoding.
    #[inline]
    pub fn as_serialized(&self) -> SerializedEntry<'_, 'a, Cx> {
        SerializedEntry {
            id: self.data.id,
            codec: self.data.codec,
            codec_id: self.data.codec_id,
            value: Maybe::Borrowed(&self.value),
        }
    }
}

/// Serialized form of [`DataTrackerEntry`] for encoding and decoding purposes.
///
/// The decoding behavior differs from vanilla implementation as it _decodes the ID byte by its own._
pub struct SerializedEntry<'borrow, 'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    id: u32,
    codec: UnsafeCodec<'a, Cx>,
    codec_id: CodecId,
    value: Maybe<'borrow, DynValue<'a, Cx>, Box<DynValue<'a, Cx>>>,
}

impl<'a, Cx, Fw> Encode<Fw> for SerializedEntry<'_, 'a, Cx>
where
    Cx: EntityDataCx<'a>,
    Fw: ForwardToWithLocalCx<Forwarded: BufMut, LocalCx = Cx::LocalContext<'a>>,
{
    fn encode(&self, buf: Fw) -> Result<(), edcode2::BoxedError<'static>> {
        let forwarded = buf.forward();
        let local_cx = forwarded.local_cx;
        let mut buf = forwarded.inner;

        if self.id > MAX_DATA_ID {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "invalid data id: expected <= {MAX_DATA_ID}, got {}",
                    self.id
                ),
            )
            .into());
        }
        buf.put_u8(self.id as u8);
        self.codec_id.encode(&mut buf)?;
        (self.codec.encode)(&*self.value, &mut buf, local_cx)
    }
}

impl<'a, 'de, Cx, Fw> Decode<'de, Fw> for SerializedEntry<'static, 'a, Cx>
where
    Cx: EntityDataCx<'a>,
    Fw: ForwardToWithLocalCx<Forwarded: Buf, LocalCx = Cx::LocalContext<'a>>,
    Cx::LocalContext<'a>: LocalContext<&'a CodecRegistry<'a, Cx>>,
{
    fn decode(buf: Fw) -> Result<Self, edcode2::BoxedError<'de>> {
        let forwarded = buf.forward();
        let local_cx = forwarded.local_cx;
        let mut buf = forwarded.inner;

        let id = buf.get_u8() as u32;
        if id > MAX_DATA_ID {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid data id: expected <= {MAX_DATA_ID}, got {id}"),
            )
            .into());
        }
        let codec_id = CodecId::decode(&mut buf)?;
        let codec = local_cx.acquire().get_raw(codec_id).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("codec with id {} not found in registry", codec_id.0),
            )
        })?;

        (codec.decode)(&mut buf, local_cx).map(|value| Self {
            id,
            codec,
            codec_id,
            value: Maybe::Owned(value),
        })
    }
}

impl<'a, Cx> Debug for SerializedEntry<'_, 'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializedEntry")
            .field("id", &self.id)
            .field("codec", &self.codec)
            .finish_non_exhaustive()
    }
}

impl<'a, Cx> Clone for SerializedEntry<'_, 'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            codec: self.codec,
            codec_id: self.codec_id,
            value: self.value.clone(),
        }
    }
}

impl<'a, T: ?Sized + Debug, Cx> Debug for DataTrackerEntry<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTrackerEntry")
            .field("data", &self.data.id)
            .field("value", &&self.value)
            .field("dirty", &self.dirty)
            .finish_non_exhaustive()
    }
}

impl<'a, Cx> Debug for RawTrackedData<'a, Cx>
where
    Cx: EntityDataCx<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TrackedData")
            .field(&self.id)
            .finish_non_exhaustive()
    }
}

impl<'a, T, Cx> Debug for TrackedData<'a, T, Cx>
where
    Cx: EntityDataCx<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.raw, f)
    }
}

impl<'a, Cx: EntityDataCx<'a>> Debug for DataTracker<'a, Cx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTracker")
            .field(
                "entries",
                &self.entries.iter().map(|e| e.data.id).collect::<Vec<_>>(),
            )
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<'a, Cx: EntityDataCx<'a>> Debug for DataTrackerBuilder<'a, Cx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTrackerBuilder")
            .field(
                "entries",
                &self
                    .entries
                    .iter()
                    .filter_map(|e| e.as_ref().map(|e| e.data.id))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
