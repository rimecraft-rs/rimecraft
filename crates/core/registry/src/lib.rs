//! Registry related stuffs used to register various
//! in-game components.
//!
//! Registry system allows the game to enumerate all known types of
//! something, and to assign a unique identifier to each of those.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::{Deref, Index},
    sync::OnceLock,
};

use entry::RefEntry;
use key::Key;
use parking_lot::RwLock;
use tag::Tags;

pub mod entry;
pub mod key;
pub mod tag;

#[doc(alias = "Holder")]
pub use entry::Entry as RegistryEntry;
#[doc(alias = "ResourceKey")]
pub use key::Key as RegistryKey;
pub use tag::TagKey;

/// Immutable registry of various in-game components.
#[derive(Debug)]
pub struct Registry<K, T> {
    key: Key<K, Self>,

    entries: Vec<RefEntry<K, T>>,
    kv: HashMap<K, usize>,
    tv: RwLock<HashMap<TagKey<K, T>, Vec<usize>>>,

    /// The default registration raw id.
    default: Option<usize>,
}

/// Reference of a registration.
///
/// # Serialization and Deserialization
///
/// This type can be serialized and deserialized using `serde` and `edcode`.
/// (with `serde` feature and `edcode` feature respectively)
///
/// ## Serde
///
/// When serializing this reference with `serde`, it will serialize the ID
/// of the entry, if the serializer is **human readable**. Otherwise, it will
/// serialize the **raw ID** of the entry.
///
/// This corresponds to the `compressed` option in *Mojang Serialization*.
pub struct Reg<'a, K, T> {
    raw: usize,
    registry: &'a Registry<K, T>,
    value: &'a T,
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
        let value = self.entries[index].value()?;
        Some(Reg {
            raw: index,
            registry: self,
            value,
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
        let value = &self.entries.get(raw)?.value()?;
        Some(Reg {
            raw,
            registry: self,
            value,
        })
    }

    /// Gets all entries of this registry.
    #[inline]
    pub fn entries(&self) -> Entries<'_, K, T> {
        Entries {
            inner: EntriesInner::Direct {
                registry: self,
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

impl<K: std::fmt::Debug, T: std::fmt::Debug> std::fmt::Debug for Reg<'_, K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegRef")
            .field("raw", &self.raw)
            .field("registry", &self.registry.key)
            .field("value", &self.value)
            .finish()
    }
}

impl<'a, K, T> Reg<'a, K, T> {
    /// Gets the inner reference of this reference.
    #[inline]
    pub fn into_inner(this: Self) -> &'a T {
        this.value
    }

    /// Gets the raw index of this reference.
    #[inline]
    pub fn raw_id(this: Self) -> usize {
        this.raw
    }

    /// Gets the registry of this reference.
    #[inline]
    pub fn registry(this: Self) -> &'a Registry<K, T> {
        this.registry
    }
}

impl<'a, K, T> From<Reg<'a, K, T>> for &'a RefEntry<K, T> {
    #[inline]
    fn from(value: Reg<'a, K, T>) -> Self {
        &value.registry.entries[value.raw]
    }
}

impl<K, T> AsRef<RefEntry<K, T>> for Reg<'_, K, T> {
    #[inline]
    fn as_ref(&self) -> &RefEntry<K, T> {
        &self.registry.entries[self.raw]
    }
}

impl<K, T> PartialEq<T> for Reg<'_, K, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.value == other
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
        self.value
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

impl<'r, K, T> Default for Reg<'r, K, T>
where
    K: 'r,
    T: ProvideRegistry<'r, K, T> + 'r,
{
    #[inline]
    fn default() -> Self {
        T::registry()
            .default_entry()
            .expect("default entry not found in registry")
    }
}

/// Trait for converting to a key.
pub trait AsKey<K, T> {
    /// Converts to a key.
    fn as_key<'a>(&'a self, registry: &'a Key<K, Registry<K, T>>) -> &'a K;
}

impl<K, T> AsKey<K, T> for K {
    #[inline(always)]
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
        registry: &'a Registry<K, T>,
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
            EntriesInner::Direct { registry, iter } => iter.next().and_then(|(raw, entry)| {
                entry.value().map(|value| Reg {
                    raw,
                    registry,
                    value,
                })
            }),
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
}

impl<K, T> RegistryMut<K, T> {
    /// Creates a new mutable registry.
    #[inline]
    pub const fn new(key: Key<K, Registry<K, T>>) -> Self {
        Self {
            key,
            entries: Vec::new(),
            keys: OnceLock::new(),
            default: None,
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
    pub fn register(&mut self, key: Key<K, T>, value: T) -> Result<usize, (Key<K, T>, T)> {
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
        let id = self.register(key, value)?;
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
        Registry {
            key: value.key,
            kv: entries
                .iter()
                .enumerate()
                .map(|(raw, entry)| (entry.key.value().clone(), raw))
                .collect(),
            tv: RwLock::new(HashMap::new()),
            entries,
            default: value.default,
        }
    }
}

/// Trait for providing a registry from reference.
pub trait ProvideRegistryRef<'r, K, T> {
    /// Gets the registry from reference.
    fn registry_ref(&self) -> &'r Registry<K, T>;
}

/// Trait for providing a registry.
pub trait ProvideRegistry<'r, K, T> {
    /// Gets the registry.
    fn registry() -> &'r Registry<K, T>;
}

impl<'r, K, T, S> ProvideRegistryRef<'r, K, T> for S
where
    S: ProvideRegistry<'r, K, T>,
{
    #[inline]
    fn registry_ref(&self) -> &'r Registry<K, T> {
        Self::registry()
    }
}

impl<K, T> Registry<K, T>
where
    K: Hash + Eq + Clone,
{
    /// Binds given tags to entries, and
    /// removes old tag bindings.
    #[doc(alias = "populate_tags")]
    pub fn bind_tags<'a, I>(&'a self, entries: I)
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
    //! Helper module for `serde` support.

    use std::hash::Hash;

    use crate::{entry::RefEntry, ProvideRegistry, Reg};

    impl<K, T> serde::Serialize for Reg<'_, K, T>
    where
        K: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            if serializer.is_human_readable() {
                let entry: &RefEntry<_, _> = self.as_ref();
                entry.key.value().serialize(serializer)
            } else {
                serializer.serialize_i32(self.raw as i32)
            }
        }
    }

    impl<'a, 'r, 'de, K, T> serde::Deserialize<'de> for Reg<'a, K, T>
    where
        'r: 'a,
        T: ProvideRegistry<'r, K, T> + 'r,
        K: serde::Deserialize<'de> + Hash + Eq + 'r,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                let key = K::deserialize(deserializer)?;
                T::registry()
                    .get(&key)
                    .ok_or_else(|| serde::de::Error::custom(format!("key not found")))
            } else {
                let raw = i32::deserialize(deserializer)? as usize;
                T::registry()
                    .of_raw(raw)
                    .ok_or_else(|| serde::de::Error::custom(format!("raw id {raw} not found")))
            }
        }
    }
}

#[cfg(feature = "edcode")]
pub mod edcode {
    //! Helper module for `edcode` support.

    use rimecraft_edcode::{Decode, Encode, VarI32};

    use crate::{ProvideRegistry, Reg};

    impl<K, T> Encode for Reg<'_, K, T> {
        #[inline]
        fn encode<B>(&self, buf: B) -> Result<(), std::io::Error>
        where
            B: rimecraft_edcode::bytes::BufMut,
        {
            VarI32(self.raw as i32).encode(buf)
        }
    }

    impl<'a, 'r, K, T> Decode for Reg<'a, K, T>
    where
        'r: 'a,
        K: 'r,
        T: ProvideRegistry<'r, K, T> + 'r,
    {
        fn decode<B>(buf: B) -> Result<Self, std::io::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            let id = VarI32::decode(buf)?.0 as usize;
            T::registry()
                .of_raw(id)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid id"))
        }
    }
}

#[allow(dead_code)]
type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[cfg(feature = "vanilla-identifier")]
/// Vanilla root registry key.
pub const VANILLA_ROOT_KEY: rimecraft_identifier::vanilla::Identifier =
    rimecraft_identifier::vanilla::Identifier::new(
        rimecraft_identifier::vanilla::MINECRAFT,
        rimecraft_identifier::vanilla::Path::new_unchecked("root"),
    );

#[cfg(feature = "vanilla-identifier")]
impl crate::key::Root for rimecraft_identifier::vanilla::Identifier {
    #[inline(always)]
    fn root() -> Self {
        VANILLA_ROOT_KEY
    }
}

#[cfg(feature = "vanilla-identifier")]
#[doc = "`Registry` using vanilla `Identifier`."]
pub type VanillaRegistry<T> = Registry<rimecraft_identifier::vanilla::Identifier, T>;

#[cfg(feature = "vanilla-identifier")]
#[doc = "Mutable `Registry` using vanilla `Identifier`."]
pub type VanillaRegistryMut<T> = RegistryMut<rimecraft_identifier::vanilla::Identifier, T>;

#[cfg(test)]
mod tests;
