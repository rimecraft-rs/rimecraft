use std::{any::TypeId, borrow::Borrow, fmt::Debug, hash::Hash, sync::Arc};

use ahash::AHashSet;

use crate::{key::Key, Registry};

type RegistryObj<'a, K> = dyn DynRegistry<K> + Send + Sync + 'a;

/// Object-safe registry marker trait.
#[doc(hidden)]
pub trait DynRegistry<K>: sealed::DynRegistrySealed<K> {}

mod sealed {
    use super::*;

    pub trait DynRegistrySealed<K> {
        fn erased_key(&self) -> &Key<K, ()>;

        fn type_id(&self) -> TypeId;
    }
}

impl<K, T> sealed::DynRegistrySealed<K> for Registry<K, T> {
    #[inline]
    fn erased_key(&self) -> &Key<K, ()> {
        self.key.cast_ref()
    }

    #[inline]
    fn type_id(&self) -> TypeId {
        typeid::of::<T>()
    }
}

struct RegCell<'a, K>(Arc<RegistryObj<'a, K>>);

impl<K> Debug for RegCell<'_, K>
where
    K: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0.erased_key())
    }
}

impl<K> Borrow<Key<K, ()>> for RegCell<'_, K> {
    #[inline]
    fn borrow(&self) -> &Key<K, ()> {
        self.0.erased_key()
    }
}

impl<K> Hash for RegCell<'_, K>
where
    K: Hash,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.erased_key().hash(state)
    }
}

impl<K> PartialEq for RegCell<'_, K>
where
    K: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.erased_key() == other.0.erased_key()
    }
}

impl<K> Eq for RegCell<'_, K> where K: Eq {}

/// A dynamic static registry manager.
#[doc(alias = "DynamicRegistryManager")]
#[derive(Debug)]
pub struct DynRegistries<'a, K> {
    map: AHashSet<RegCell<'a, K>>,
}

impl<K> DynRegistries<'_, K>
where
    K: Hash + Eq,
{
    /// Obtains a registry from the manager.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn get<T>(&self, key: &Key<K, Registry<K, T>>) -> Option<&Registry<K, T>> {
        self.map
            .get(key.cast_ref::<()>())
            .filter(|reg| (*reg.0).type_id() == typeid::of::<T>())
            .map(|reg| unsafe { &*std::ptr::from_ref(&*reg.0).cast::<Registry<K, T>>() })
    }
}

impl<'a, K> FromIterator<Arc<RegistryObj<'a, K>>> for DynRegistries<'a, K>
where
    K: Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = Arc<RegistryObj<'a, K>>>>(iter: T) -> Self {
        Self {
            map: iter.into_iter().map(|reg| RegCell(reg)).collect(),
        }
    }
}
