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

/// Immutable registry of various in-game components.
#[derive(Debug)]
pub struct Registry<K, T> {
    key: Key<K, Self>,

    entries: Vec<RefEntry<K, T>>,
    kv: HashMap<K, usize>,
    tv: RwLock<HashMap<TagKey<K, T>, Vec<usize>>>,

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
pub struct Reg<'a, K, T> {
    raw: usize,
    entry: &'a RefEntry<K, T>,
}

impl<K, T> Registry<K, T>
where
    K: Hash + Eq,
{
    /// Gets an entry with the given key.
    pub fn get<'a, Q>(&'a self, key: &Q) -> Option<Reg<'a, K, T>>
    where
        Q: AsKey<K, T>,
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
        Q: AsKey<K, T>,
    {
        self.kv.contains_key(key.as_key(&self.key))
    }

    /// Gets entries of given tag.
    pub fn of_tag<'a>(&'a self, tag: &TagKey<K, T>) -> OfTag<'a, K, T> {
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

impl<K, T> Registry<K, T> {
    /// Gets the key of this registry.
    #[inline]
    pub fn key(&self) -> &Key<K, Self> {
        &self.key
    }

    /// Gets entry of given raw id.
    pub fn of_raw(&self, raw: usize) -> Option<Reg<'_, K, T>> {
        let entry = self.entries.get(raw)?;
        debug_assert!(entry.value.is_some(), "entry is empty");

        Some(Reg { raw, entry })
    }

    /// Gets all entries of this registry.
    #[inline]
    pub fn entries(&self) -> Entries<'_, K, T> {
        Entries {
            inner: EntriesInner::Direct {
                iter: self.entries.iter().enumerate(),
            },
        }
    }

    /// Gets all values of this registry.
    #[inline]
    pub fn values(&self) -> Values<'_, K, T> {
        Values {
            inner: self.entries.iter(),
        }
    }

    /// Gets tags of this registry.
    #[inline]
    pub fn tags(&self) -> Tags<'_, K, T> {
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
    pub fn default_entry(&self) -> Option<Reg<'_, K, T>> {
        self.default.and_then(|raw| self.of_raw(raw))
    }
}

#[cfg(feature = "marking")]
impl<K, T> Registry<K, T> {
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

impl<K, T, Q> Index<Q> for Registry<K, T>
where
    K: Hash + Eq,
    Q: AsKey<K, T>,
{
    type Output = T;

    fn index(&self, index: Q) -> &Self::Output {
        self.entries[*self.kv.get(index.as_key(&self.key)).unwrap()]
            .value()
            .unwrap()
    }
}

impl<K: std::fmt::Debug, T> std::fmt::Debug for Reg<'_, K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Self::to_id(*self))
    }
}

impl<'a, K, T> Reg<'a, K, T> {
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
    pub fn registry(this: Self) -> &'a Registry<K, T> {
        let _ = this;
        unreachable!("deprecated function")
    }

    /// Gets the ID of this registration.
    #[inline]
    pub fn to_id(this: Self) -> &'a K {
        Self::to_entry(this).key().value()
    }

    /// Gets the reference entry of this registration.
    #[inline]
    pub fn to_entry(this: Self) -> &'a RefEntry<K, T> {
        this.entry
    }
}

impl<K, T> PartialEq<T> for Reg<'_, K, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        Self::to_value(*self) == other
    }
}

impl<K, T> Copy for Reg<'_, K, T> {}

impl<K, T> Clone for Reg<'_, K, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, T> Deref for Reg<'_, K, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Self::to_value(*self)
    }
}

impl<K, T> Hash for Reg<'_, K, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

impl<K, T> PartialEq for Reg<'_, K, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<K, T> Eq for Reg<'_, K, T> {}

impl<K, T> Display for Reg<'_, K, T>
where
    K: Display,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::to_id(*self))
    }
}

/// Trait for converting to a key.
pub trait AsKey<K, T> {
    /// Converts to a key.
    fn as_key<'a>(&'a self, registry: &'a Key<K, Registry<K, T>>) -> &'a K;
}

impl<K, T> AsKey<K, T> for K {
    #[inline]
    fn as_key<'a>(&'a self, _registry: &'a Key<K, Registry<K, T>>) -> &'a K {
        self
    }
}

impl<K, T> AsKey<K, T> for Key<K, T>
where
    K: PartialEq,
{
    #[inline]
    fn as_key<'a>(&'a self, registry: &'a Key<K, Registry<K, T>>) -> &'a K {
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
#[derive(Debug)]
pub struct Entries<'a, K, T> {
    inner: EntriesInner<'a, K, T>,
}

#[derive(Debug)]
enum EntriesInner<'a, K, T> {
    Direct {
        iter: std::iter::Enumerate<std::slice::Iter<'a, RefEntry<K, T>>>,
    },
    Raw {
        registry: &'a Registry<K, T>,
        iter: std::slice::Iter<'a, usize>,
    },
}

impl<'a, K, T> Iterator for Entries<'a, K, T> {
    type Item = Reg<'a, K, T>;

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

/// Iterator of entry references of a tag.
#[derive(Debug)]
pub struct OfTag<'a, K, T> {
    registry: &'a Registry<K, T>,
    inner: std::vec::IntoIter<usize>,
}

impl<'a, K, T> Iterator for OfTag<'a, K, T> {
    type Item = Reg<'a, K, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|i| self.registry.of_raw(i))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Iterator of entry values.
#[derive(Debug)]
pub struct Values<'a, K, T> {
    inner: std::slice::Iter<'a, RefEntry<K, T>>,
}

impl<'a, K, T> Iterator for Values<'a, K, T> {
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

impl<'a, K, T> IntoIterator for &'a Registry<K, T> {
    type Item = &'a T;

    type IntoIter = Values<'a, K, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Values {
            inner: self.entries.iter(),
        }
    }
}

/// Mutable registry of various in-game components.
#[derive(Debug)]
pub struct RegistryMut<K, T> {
    key: Key<K, Registry<K, T>>,
    entries: Vec<(T, RefEntry<K, T>)>,
    keys: OnceLock<HashSet<K>>,

    default: Option<usize>,

    #[cfg(all(feature = "marking", not(feature = "marking-leaked")))]
    marker: marking::PtrMarker,
    #[cfg(all(feature = "marking", feature = "marking-leaked"))]
    marker: marking::LeakedPtrMarker,
}

impl<K, T> RegistryMut<K, T> {
    /// Creates a new mutable registry.
    #[inline]
    pub fn new(key: Key<K, Registry<K, T>>) -> Self {
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
    pub fn key(&self) -> &Key<K, Registry<K, T>> {
        &self.key
    }
}

impl<K, T> RegistryMut<K, T>
where
    K: Hash + Eq + Clone,
{
    /// Registers a new entry and returns its raw id if successful.
    ///
    /// # Errors
    ///
    /// Returns back the given key and value if registration with the key already exists.
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn register(&mut self, key: Key<K, T>, value: T) -> Result<usize, (Key<K, T>, T)> {
        self.register_raw(key, value, false)
    }

    fn register_raw(
        &mut self,
        key: Key<K, T>,
        value: T,
        is_default: bool,
    ) -> Result<usize, (Key<K, T>, T)> {
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
    pub fn register_default(&mut self, key: Key<K, T>, value: T) -> Result<usize, (Key<K, T>, T)> {
        if self.default.is_some() {
            return Err((key, value));
        }
        let id = self.register_raw(key, value, true)?;
        self.default = Some(id);
        Ok(id)
    }
}

impl<K, T> From<RegistryMut<K, T>> for Registry<K, T>
where
    K: Hash + Eq + Clone,
{
    fn from(value: RegistryMut<K, T>) -> Self {
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

/// Trait for providing a registry.
#[deprecated = "use local-cx to obtain registry instead"]
pub trait ProvideRegistry<'r, K, T> {
    /// Gets the registry.
    fn registry() -> &'r Registry<K, T>;
}

impl<K, T> Registry<K, T>
where
    K: Hash + Eq + Clone,
{
    /// Binds given tags to entries, and removes old tag bindings.
    #[doc(alias = "bind_tags")]
    pub fn populate_tags<'a, I>(&'a self, entries: I)
    where
        I: IntoIterator<Item = (TagKey<K, T>, Vec<&'a RefEntry<K, T>>)>,
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

    use crate::{Reg, Registry};

    impl<K, T> serde::Serialize for Reg<'_, K, T>
    where
        K: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Self::to_entry(*self).serialize(serializer)
        }
    }

    impl<'a, 'de, K, T, Cx> DeserializeWithCx<'de, Cx> for Reg<'a, K, T>
    where
        K: DeserializeWithCx<'de, Cx> + Hash + Eq + 'a,
        Cx: LocalContext<&'a Registry<K, T>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, Cx>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let cx = deserializer.local_cx;
            let key = K::deserialize_with_cx(deserializer)?;
            cx.acquire()
                .get(&key)
                .ok_or_else(|| serde::de::Error::custom("key not found"))
        }
    }
}

#[cfg(feature = "edcode")]
mod edcode {

    use edcode2::{Buf, BufExt as _, BufMut, BufMutExt as _, Decode, Encode};
    use local_cx::{ForwardToWithLocalCx, LocalContext, WithLocalCx};

    use crate::{Reg, Registry};

    impl<K, T, B> Encode<B> for Reg<'_, K, T>
    where
        B: BufMut,
    {
        #[inline]
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            buf.put_variable(self.raw as u32);
            Ok(())
        }
    }

    impl<'a, 'r, 'de, K: 'r, T: 'r, Fw> Decode<'de, Fw> for Reg<'a, K, T>
    where
        'r: 'a,
        Fw: ForwardToWithLocalCx<Forwarded: Buf>,
        Fw::LocalCx: LocalContext<&'r Registry<K, T>>,
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

#[allow(dead_code)]
type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[cfg(test)]
mod tests;
