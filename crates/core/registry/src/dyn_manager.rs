use std::{any::TypeId, borrow::Borrow, fmt::Debug, hash::Hash, sync::Arc};

use ahash::AHashSet;

use crate::{Registry, RegistryCx, key::Key};

type RegistryObj<'a, Cx> = dyn DynRegistry<Cx> + Send + Sync + 'a;

/// Object-safe registry marker trait.
#[doc(hidden)]
pub trait DynRegistry<Cx>: sealed::DynRegistrySealed<Cx>
where
    Cx: RegistryCx,
{
}

mod sealed {
    use super::*;

    pub trait DynRegistrySealed<Cx>
    where
        Cx: RegistryCx,
    {
        fn erased_key(&self) -> &Key<(), Cx>;

        fn type_id(&self) -> TypeId;
    }
}

impl<T, Cx> sealed::DynRegistrySealed<Cx> for Registry<T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn erased_key(&self) -> &Key<(), Cx> {
        self.key.cast_ref()
    }

    #[inline]
    fn type_id(&self) -> TypeId {
        typeid::of::<T>()
    }
}

struct RegCell<'a, Cx>(Arc<RegistryObj<'a, Cx>>)
where
    Cx: RegistryCx;

impl<Cx> Debug for RegCell<'_, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0.erased_key())
    }
}

impl<Cx> Borrow<Key<(), Cx>> for RegCell<'_, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn borrow(&self) -> &Key<(), Cx> {
        self.0.erased_key()
    }
}

impl<Cx> Hash for RegCell<'_, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.erased_key().hash(state)
    }
}

impl<Cx> PartialEq for RegCell<'_, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.erased_key() == other.0.erased_key()
    }
}

impl<Cx> Eq for RegCell<'_, Cx> where Cx: RegistryCx {}

/// A dynamic static registry manager.
#[doc(alias = "DynamicRegistryManager")]
pub struct DynRegistries<'a, Cx>
where
    Cx: RegistryCx,
{
    map: AHashSet<RegCell<'a, Cx>>,
}

impl<Cx> DynRegistries<'_, Cx>
where
    Cx: RegistryCx,
{
    /// Obtains a registry from the manager.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn get<T>(&self, key: &Key<Registry<T, Cx>, Cx>) -> Option<&Registry<T, Cx>> {
        self.map
            .get(key.cast_ref::<()>())
            .filter(|reg| (*reg.0).type_id() == typeid::of::<T>())
            .map(|reg| unsafe { &*std::ptr::from_ref(&*reg.0).cast::<Registry<T, Cx>>() })
    }
}

impl<'a, Cx> FromIterator<Arc<RegistryObj<'a, Cx>>> for DynRegistries<'a, Cx>
where
    Cx: RegistryCx,
{
    fn from_iter<T: IntoIterator<Item = Arc<RegistryObj<'a, Cx>>>>(iter: T) -> Self {
        Self {
            map: iter.into_iter().map(|reg| RegCell(reg)).collect(),
        }
    }
}

impl<Cx> Debug for DynRegistries<'_, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynRegistries")
            .field("map", &self.map)
            .finish()
    }
}
