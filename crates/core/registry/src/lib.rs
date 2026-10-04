//! Registry related stuffs used to register various
//! in-game components.
//!
//! Registry system allows the game to enumerate all known types of
//! something, and to assign a unique identifier to each of those.

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    ops::{Deref, Index},
    sync::OnceLock,
};

use entry::RefEntry;
use global_cx::ProvideIdTy;
use key::Key;
use parking_lot::RwLock;
use tag::Tags;

mod dyn_manager;
pub mod entry;
pub mod key;
pub mod tag;

#[doc(alias = "Holder")]
pub use entry::Entry as RegistryEntry;
#[doc(alias = "ResourceKey")]
pub use key::Key as RegistryKey;
pub use tag::TagKey;

pub use dyn_manager::*;

/// Global context types for registries.
pub trait RegistryCx: ProvideIdTy {
    // make this a standalone trait for future extensions (e.g. hasher)
}

/// Immutable registry of various in-game components.
pub struct Registry<T, Cx>
where
    Cx: RegistryCx,
{
    key: Key<Self, Cx>,

    entries: Vec<RefEntry<T, Cx>>,
    kv: HashMap<Cx::Id, usize>,
    tv: RwLock<HashMap<TagKey<T, Cx>, Vec<usize>>>,

    /// The default registration raw id.
    default: Option<usize>,

    #[cfg(all(feature = "marking", not(feature = "marking-leaked")))]
    marker: marking::PtrMarker,
    #[cfg(all(feature = "marking", feature = "marking-leaked"))]
    marker: marking::LeakedPtrMarker,
}

/// Reference of a registration.
///
/// When serializing this reference with `serde`, it will serialize the ID
/// of the entry.
pub struct Reg<'a, T, Cx>
where
    Cx: RegistryCx,
{
    raw: usize,
    entry: &'a RefEntry<T, Cx>,
}

impl<T, Cx> Registry<T, Cx>
where
    Cx: RegistryCx,
{
    /// Gets an entry with the given key.
    pub fn get<'a, Q>(&'a self, key: &Q) -> Option<Reg<'a, T, Cx>>
    where
        Q: AsKey<T, Cx>,
    {
        let index = *self.kv.get(key.as_key(&self.key))?;

        let entry = &self.entries[index];
        debug_assert!(entry.value.is_some(), "entry is empty");

        Some(Reg {
            raw: index,
            entry: &self.entries[index],
        })
    }

    /// Whether this registry contains the given key.
    #[inline]
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: AsKey<T, Cx>,
    {
        self.kv.contains_key(key.as_key(&self.key))
    }

    /// Gets entries of given tag.
    pub fn of_tag<'a>(&'a self, tag: &TagKey<T, Cx>) -> OfTag<'a, T, Cx> {
        OfTag {
            inner: self
                .tv
                .read()
                .get(tag)
                .cloned()
                .unwrap_or_default()
                .into_iter(),
            registry: self,
        }
    }
}

impl<T, Cx> Registry<T, Cx>
where
    Cx: RegistryCx,
{
    /// Gets the key of this registry.
    #[inline]
    pub fn key(&self) -> &Key<Self, Cx> {
        &self.key
    }

    /// Gets entry of given raw id.
    pub fn of_raw(&self, raw: usize) -> Option<Reg<'_, T, Cx>> {
        let entry = self.entries.get(raw)?;
        debug_assert!(entry.value.is_some(), "entry is empty");

        Some(Reg { raw, entry })
    }

    /// Gets all entries of this registry.
    #[inline]
    pub fn entries(&self) -> Entries<'_, T, Cx> {
        Entries {
            inner: EntriesInner::Direct {
                iter: self.entries.iter().enumerate(),
            },
        }
    }

    /// Gets all values of this registry.
    #[inline]
    pub fn values(&self) -> Values<'_, T, Cx> {
        Values {
            inner: self.entries.iter(),
        }
    }

    /// Gets tags of this registry.
    #[inline]
    pub fn tags(&self) -> Tags<'_, T, Cx> {
        Tags {
            inner: self.tv.read(),
            registry: self,
        }
    }

    /// Gets the number of entries in this registry.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if this registry is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gets the default entry of this registry.
    #[inline]
    pub fn default_entry(&self) -> Option<Reg<'_, T, Cx>> {
        self.default.and_then(|raw| self.of_raw(raw))
    }
}

#[cfg(feature = "marking")]
impl<T, Cx> Registry<T, Cx>
where
    Cx: RegistryCx,
{
    /// Gets the marker of this registry.
    #[inline]
    pub fn marker(&self) -> &marking::PtrMarker {
        #[cfg(feature = "marking-leaked")]
        return self.marker.as_non_leaked();

        #[cfg(not(feature = "marking-leaked"))]
        return &self.marker;
    }

    /// Gets the leaked marker of this registry.
    #[inline]
    #[cfg(feature = "marking-leaked")]
    pub fn marker_leaked(&self) -> marking::LeakedPtrMarker {
        self.marker
    }
}

impl<T, Cx, Q> Index<Q> for Registry<T, Cx>
where
    Cx: RegistryCx,
    Q: AsKey<T, Cx>,
{
    type Output = T;

    fn index(&self, index: Q) -> &Self::Output {
        self.entries[*self.kv.get(index.as_key(&self.key)).unwrap()]
            .value()
            .unwrap()
    }
}

impl<T, Cx> std::fmt::Debug for Registry<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry")
            .field("key", &self.key)
            .field("entries", &self.entries)
            .field("kv", &self.kv)
            .field("tv", &self.tv)
            .field("default", &self.default)
            .finish_non_exhaustive()
    }
}

impl<T, Cx> std::fmt::Debug for Reg<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Self::to_id(*self))
    }
}

impl<'a, T, Cx> Reg<'a, T, Cx>
where
    Cx: RegistryCx,
{
    /// Gets the inner reference of this reference.
    #[inline]
    pub fn to_value(this: Self) -> &'a T {
        unsafe { this.entry.value().unwrap_unchecked() }
    }

    /// Gets the raw index of this reference.
    #[inline]
    pub fn to_raw_id(this: Self) -> usize {
        this.raw
    }

    /// Gets the registry of this reference.
    #[deprecated = "this function fails"]
    pub fn registry(this: Self) -> &'a Registry<T, Cx> {
        let _ = this;
        unreachable!("deprecated function")
    }

    /// Gets the ID of this registration.
    #[inline]
    pub fn to_id(this: Self) -> &'a Cx::Id {
        Self::to_entry(this).key().value()
    }

    /// Gets the reference entry of this registration.
    #[inline]
    pub fn to_entry(this: Self) -> &'a RefEntry<T, Cx> {
        this.entry
    }
}

impl<T, Cx> PartialEq<T> for Reg<'_, T, Cx>
where
    Cx: RegistryCx,
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        Self::to_value(*self) == other
    }
}

impl<T, Cx> Copy for Reg<'_, T, Cx> where Cx: RegistryCx {}

impl<T, Cx> Clone for Reg<'_, T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Cx> Deref for Reg<'_, T, Cx>
where
    Cx: RegistryCx,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Self::to_value(*self)
    }
}

impl<T, Cx> Hash for Reg<'_, T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

impl<T, Cx> PartialEq for Reg<'_, T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T, Cx> Eq for Reg<'_, T, Cx> where Cx: RegistryCx {}

impl<T, Cx> Display for Reg<'_, T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::to_id(*self))
    }
}

/// Trait for converting to a key.
///
/// Use [`Query`] if you found the identifier type hasn't implemented this.
pub trait AsKey<T, Cx>
where
    Cx: RegistryCx,
{
    /// Converts to a key.
    fn as_key<'a>(&'a self, registry: &'a Key<Registry<T, Cx>, Cx>) -> &'a Cx::Id;
}

/// A newtype wrapper around identifier type, for [`AsKey`] implementation.
///
/// Implementors of global contexts can also implement `AsKey` for the identifier type,
/// so this type can be retired. They are not implemented automatically due to restrictions
/// of the Rust compiler.
#[derive(Debug, Clone, Copy)]
pub struct Query<K>(pub K);

impl<T, Cx> AsKey<T, Cx> for Query<&Cx::Id>
where
    Cx: RegistryCx,
{
    #[inline]
    fn as_key<'a>(&'a self, _registry: &'a Key<Registry<T, Cx>, Cx>) -> &'a Cx::Id {
        self.0
    }
}

impl<T, Cx> AsKey<T, Cx> for Key<T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn as_key<'a>(&'a self, registry: &'a Key<Registry<T, Cx>, Cx>) -> &'a Cx::Id {
        if self.registry() == registry.value() {
            self.value()
        } else {
            panic! {
                "RegistryKey could not convert to key properly: not from given registry reference"
            }
        }
    }
}

/// Iterator of entry references.
pub struct Entries<'a, T, Cx>
where
    Cx: RegistryCx,
{
    inner: EntriesInner<'a, T, Cx>,
}

enum EntriesInner<'a, T, Cx>
where
    Cx: RegistryCx,
{
    Direct {
        iter: std::iter::Enumerate<std::slice::Iter<'a, RefEntry<T, Cx>>>,
    },
    Raw {
        registry: &'a Registry<T, Cx>,
        iter: std::slice::Iter<'a, usize>,
    },
}

impl<'a, T, Cx> Iterator for Entries<'a, T, Cx>
where
    Cx: RegistryCx,
{
    type Item = Reg<'a, T, Cx>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            EntriesInner::Direct { iter } => iter.next().map(|(raw, entry)| Reg { raw, entry }),
            EntriesInner::Raw { registry, iter } => {
                iter.next().and_then(|raw| registry.of_raw(*raw))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            EntriesInner::Direct { iter, .. } => iter.size_hint(),
            EntriesInner::Raw { iter, .. } => iter.size_hint(),
        }
    }
}

impl<T, Cx> std::fmt::Debug for Entries<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Entries").field(&self.inner).finish()
    }
}

impl<T, Cx> std::fmt::Debug for EntriesInner<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct { iter } => f.debug_struct("Direct").field("iter", iter).finish(),
            Self::Raw { registry: _, iter } => f
                .debug_struct("Raw")
                .field("iter", iter)
                .finish_non_exhaustive(),
        }
    }
}

/// Iterator of entry references of a tag.
pub struct OfTag<'a, T, Cx>
where
    Cx: RegistryCx,
{
    registry: &'a Registry<T, Cx>,
    inner: std::vec::IntoIter<usize>,
}

impl<'a, T, Cx> Iterator for OfTag<'a, T, Cx>
where
    Cx: RegistryCx,
{
    type Item = Reg<'a, T, Cx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|i| self.registry.of_raw(i))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T, Cx> std::fmt::Debug for OfTag<'_, T, Cx>
where
    Cx: RegistryCx,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OfTag")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

/// Iterator of entry values.
pub struct Values<'a, T, Cx>
where
    Cx: RegistryCx,
{
    inner: std::slice::Iter<'a, RefEntry<T, Cx>>,
}

impl<'a, T, Cx> Iterator for Values<'a, T, Cx>
where
    Cx: RegistryCx,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(RefEntry::value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T, Cx> std::fmt::Debug for Values<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Values").field(&self.inner).finish()
    }
}

impl<'a, T, Cx> IntoIterator for &'a Registry<T, Cx>
where
    Cx: RegistryCx,
{
    type Item = &'a T;

    type IntoIter = Values<'a, T, Cx>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Values {
            inner: self.entries.iter(),
        }
    }
}

/// Mutable registry of various in-game components.
pub struct RegistryMut<T, Cx>
where
    Cx: RegistryCx,
{
    key: Key<Registry<T, Cx>, Cx>,
    entries: Vec<(T, RefEntry<T, Cx>)>,
    keys: OnceLock<HashSet<Cx::Id>>,

    default: Option<usize>,

    #[cfg(all(feature = "marking", not(feature = "marking-leaked")))]
    marker: marking::PtrMarker,
    #[cfg(all(feature = "marking", feature = "marking-leaked"))]
    marker: marking::LeakedPtrMarker,
}

impl<T, Cx> RegistryMut<T, Cx>
where
    Cx: RegistryCx,
{
    /// Creates a new mutable registry.
    #[cfg_attr(
        feature = "marking-leaked",
        doc = "\n_Note on feature `marking-leaked`:_ This function introduces tiny memory leaking behavior."
    )]
    #[inline]
    pub fn new(key: Key<Registry<T, Cx>, Cx>) -> Self {
        Self {
            key,
            entries: Vec::new(),
            keys: OnceLock::new(),
            default: None,

            #[cfg(feature = "marking")]
            marker: Default::default(),
        }
    }

    /// Gets the key of this registry.
    #[inline]
    pub fn key(&self) -> &Key<Registry<T, Cx>, Cx> {
        &self.key
    }
}

impl<T, Cx> RegistryMut<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Clone,
{
    /// Registers a new entry and returns its raw id if successful.
    ///
    /// # Errors
    ///
    /// Returns back the given key and value if registration with the key already exists.
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn register(&mut self, key: Key<T, Cx>, value: T) -> Result<usize, (Key<T, Cx>, T)> {
        self.register_raw(key, value, false)
    }

    fn register_raw(
        &mut self,
        key: Key<T, Cx>,
        value: T,
        is_default: bool,
    ) -> Result<usize, (Key<T, Cx>, T)> {
        if self.keys.get_mut().is_none() {
            self.keys = HashSet::new().into();
        }
        let keys = self.keys.get_mut().expect("keys not initialized");
        if keys.contains(key.value()) {
            return Err((key, value));
        }
        keys.insert(key.value().clone());
        let raw = self.entries.len();
        self.entries.push((
            value,
            RefEntry {
                raw,
                key,
                value: None,
                tags: RwLock::new(HashSet::new()),
                is_default,
                #[cfg(feature = "marking-leaked")]
                marker: self.marker,
            },
        ));
        Ok(raw)
    }

    /// Registers a unique default entry of this registry.
    ///
    /// See [`Self::register`].
    #[allow(clippy::missing_errors_doc)]
    pub fn register_default(
        &mut self,
        key: Key<T, Cx>,
        value: T,
    ) -> Result<usize, (Key<T, Cx>, T)> {
        if self.default.is_some() {
            return Err((key, value));
        }
        let id = self.register_raw(key, value, true)?;
        self.default = Some(id);
        Ok(id)
    }
}

impl<T, Cx> std::fmt::Debug for RegistryMut<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistryMut")
            .field("key", &self.key)
            .field("entries", &self.entries)
            .field("keys", &self.keys)
            .field("default", &self.default)
            .finish_non_exhaustive()
    }
}

impl<T, Cx> From<RegistryMut<T, Cx>> for Registry<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Clone,
{
    fn from(value: RegistryMut<T, Cx>) -> Self {
        let entries: Vec<_> = value
            .entries
            .into_iter()
            .map(|(v, mut r)| {
                r.value = Some(v);
                r
            })
            .collect();
        Self {
            key: value.key,
            kv: entries
                .iter()
                .enumerate()
                .map(|(raw, entry)| (entry.key.value().clone(), raw))
                .collect(),
            tv: RwLock::new(HashMap::new()),
            entries,
            default: value.default,
            #[cfg(feature = "marking")]
            marker: value.marker,
        }
    }
}

impl<T, Cx> Registry<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Clone,
{
    /// Binds given tags to entries, and removes old tag bindings.
    #[doc(alias = "bind_tags")]
    pub fn populate_tags<'a, I>(&'a self, entries: I)
    where
        I: IntoIterator<Item = (TagKey<T, Cx>, Vec<&'a RefEntry<T, Cx>>)>,
    {
        self.clear_tags();

        let iter = entries.into_iter();
        let mut tv = self.tv.write();
        for (tag, entries) in iter {
            for entry in entries.iter() {
                entry.tags.write().insert(tag.clone());
            }
            if let Some(vec) = tv.get_mut(&tag) {
                vec.extend(entries.into_iter().map(|e| e.raw));
            } else {
                tv.insert(tag, entries.into_iter().map(|e| e.raw).collect());
            }
        }
    }

    /// Clears all tags.
    pub fn clear_tags(&self) {
        for entry in self.entries.iter() {
            entry.tags.write().clear();
        }
        self.tv.write().clear();
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::hash::Hash;

    use local_cx::{LocalContext, serde::DeserializeWithCx};

    use crate::{Query, Reg, Registry, RegistryCx};

    impl<T, Cx> serde::Serialize for Reg<'_, T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Self::to_entry(*self).serialize(serializer)
        }
    }

    impl<'a, 'de, T, Cx, L> DeserializeWithCx<'de, L> for Reg<'a, T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: DeserializeWithCx<'de, L> + Hash + Eq + 'a,
        L: LocalContext<&'a Registry<T, Cx>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, L>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let cx = deserializer.local_cx;
            let key = Cx::Id::deserialize_with_cx(deserializer)?;
            cx.acquire()
                .get(&Query(&key))
                .ok_or_else(|| serde::de::Error::custom("key not found"))
        }
    }
}

#[cfg(feature = "edcode")]
mod edcode {

    use edcode2::{Buf, BufExt as _, BufMut, BufMutExt as _, Decode, Encode};
    use local_cx::{ForwardToWithLocalCx, LocalContext, WithLocalCx};

    use crate::{Reg, Registry, RegistryCx};

    impl<T, Cx, B> Encode<B> for Reg<'_, T, Cx>
    where
        Cx: RegistryCx,
        B: BufMut,
    {
        #[inline]
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            buf.put_variable(self.raw as u32);
            Ok(())
        }
    }

    impl<'a, 'r, 'de, T: 'r, Cx, Fw> Decode<'de, Fw> for Reg<'a, T, Cx>
    where
        'r: 'a,
        Cx: RegistryCx,
        Fw: ForwardToWithLocalCx<Forwarded: Buf>,
        Fw::LocalCx: LocalContext<&'r Registry<T, Cx>>,
    {
        fn decode(buf: Fw) -> Result<Self, edcode2::BoxedError<'de>> {
            let WithLocalCx { inner, local_cx } = buf.forward();
            let mut buf = inner;
            let id = buf.get_variable::<i32>() as usize;
            local_cx
                .acquire()
                .of_raw(id)
                .ok_or_else(|| format!("invalid id: {id}").into())
        }
    }
}
