//! Registry related stuffs used to register various
//! in-game components.
//!
//! Registry system allows the game to enumerate all known types of
//! something, and to assign a unique identifier to each of those.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::{Deref, Index},
};

use entry::RefEntry;
use key::Key;
use parking_lot::RwLock;
use tag::{TagKey, Tags};

pub mod entry;
pub mod key;
pub mod tag;

/// Immutable registry of various in-game components.
#[derive(Debug)]
pub struct Registry<K, T> {
    key: Key<K, Self>,

    entries: Vec<RefEntry<K, T>>,
    kv: HashMap<K, usize>,
    tv: RwLock<HashMap<TagKey<K, T>, Vec<usize>>>,
}

/// Reference of a registration.
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
    keys: HashSet<Key<K, T>>,
}

impl<K, T> RegistryMut<K, T> {
    /// Creates a new mutable registry.
    #[inline]
    pub fn new(key: Key<K, Registry<K, T>>) -> Self {
        Self {
            key,
            entries: Vec::new(),
            keys: HashSet::new(),
        }
    }
}

impl<K, T> RegistryMut<K, T>
where
    K: Hash + Eq,
{
    /// Registers a new entry and returns
    /// its raw id if successful.
    ///
    /// # Errors
    ///
    /// Returns back the given key and value if
    /// registration with the key already exists.
    pub fn register(&mut self, key: Key<K, T>, value: T) -> Result<usize, (Key<K, T>, T)> {
        if self.keys.contains(&key) {
            return Err((key, value));
        }
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
pub mod serde {
    //! Helper module for `serde` support.

    use std::hash::Hash;

    use crate::{entry::RefEntry, ProvideRegistry, Reg};

    impl<K, T> serde::Serialize for Reg<'_, K, T>
    where
        K: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let entry: &RefEntry<_, _> = self.as_ref();
            entry.key.value().serialize(serializer)
        }
    }

    impl<'a, 'r, 'de, K, T> serde::Deserialize<'de> for Reg<'a, K, T>
    where
        'r: 'a,
        T: ProvideRegistry<'r, K, T> + 'r,
        K: serde::Deserialize<'de> + Hash + Eq + std::fmt::Debug + 'r,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let key = K::deserialize(deserializer)?;
            T::registry()
                .get(&key)
                .ok_or_else(|| serde::de::Error::custom(format!("key {key:?} not found")))
        }
    }

    /// Wrapper for serializing a compressed entry.
    #[derive(Debug, Clone, Copy)]
    pub struct Compressed<T>(pub T);

    impl<K, T> serde::Serialize for Compressed<&Reg<'_, K, T>>
    where
        K: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_i32(self.0.raw as i32)
        }
    }

    impl<K, T> serde::Serialize for Compressed<Reg<'_, K, T>>
    where
        K: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Compressed(&self.0).serialize(serializer)
        }
    }

    impl<'a, 'r, 'de, K, T> serde::Deserialize<'de> for Compressed<Reg<'a, K, T>>
    where
        'r: 'a,
        T: ProvideRegistry<'r, K, T> + 'r,
        K: serde::Deserialize<'de> + Hash + Eq + std::fmt::Debug + 'r,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let raw = i32::deserialize(deserializer)? as usize;
            T::registry()
                .of_raw(raw)
                .map(Compressed)
                .ok_or_else(|| serde::de::Error::custom(format!("raw id {raw} not found")))
        }
    }
}

#[cfg(feature = "edcode")]
pub mod edcode {
    //! Helper module for `edcode` support.

    use std::convert::Infallible;

    use rimecraft_edcode::{error::VarI32TooBigError, Decode, Encode, VarI32};

    use crate::{ProvideRegistry, Reg};

    /// Error type for `edcode` support.
    #[derive(Debug)]
    pub enum Error<K> {
        /// Error for invalid key.
        InvalidKey(K),
        /// Error for invalid raw id.
        InvalidRawId(usize),
        /// Error for `VarI32`.
        VarI32(VarI32TooBigError),
    }

    impl<K> std::fmt::Display for Error<K>
    where
        K: std::fmt::Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::InvalidKey(id) => write!(f, "element not found for key {id:?}"),
                Error::InvalidRawId(id) => write!(f, "element not found for raw id {id}"),
                Error::VarI32(err) => write!(f, "{err}"),
            }
        }
    }

    impl<K> std::error::Error for Error<K> where K: std::fmt::Debug {}

    impl<K> From<VarI32TooBigError> for Error<K> {
        #[inline]
        fn from(value: VarI32TooBigError) -> Self {
            Self::VarI32(value)
        }
    }

    impl<K, T> Encode for Reg<'_, K, T> {
        type Error = Infallible;

        #[inline]
        fn encode<B>(&self, buf: B) -> Result<(), Self::Error>
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
        type Output = Self;

        type Error = Error<K>;

        fn decode<B>(buf: B) -> Result<Self::Output, Self::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            let id = VarI32::decode(buf)? as usize;
            T::registry()
                .of_raw(id)
                .map(From::from)
                .ok_or_else(|| Error::InvalidRawId(id))
        }
    }
}

#[allow(dead_code)]
type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[cfg(feature = "vanilla-identifier")]
impl crate::key::Root for rimecraft_identifier::vanilla::Identifier {
    #[inline]
    fn root() -> Self {
        Self::new(
            Default::default(),
            rimecraft_identifier::vanilla::Path::new_unchecked("root"),
        )
    }
}

#[cfg(feature = "vanilla-registry")]
#[doc = "Registry using vanilla `Identifier`."]
pub type VanillaRegistry<T> = Registry<rimecraft_identifier::vanilla::Identifier, T>;
