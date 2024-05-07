//! Component map implementation.

use std::{any::TypeId, borrow::Borrow, hash::Hash};

use ahash::AHashMap;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_maybe::{Maybe, SimpleOwned};

use crate::{ComponentType, ErasedComponentType, Object, RawErasedComponentType};

#[repr(transparent)]
struct CompTyCell<'a, Cx: ProvideIdTy>(ErasedComponentType<'a, Cx>);

pub struct ComponentMap<'a, Cx>(MapInner<'a, Cx>)
where
    Cx: ProvideIdTy;

enum MapInner<'a, Cx>
where
    Cx: ProvideIdTy,
{
    Empty,
    Patched {
        base: &'a ComponentMap<'a, Cx>,
        changes: AHashMap<CompTyCell<'a, Cx>, Option<Box<Object>>>,
    },
    Simple(AHashMap<CompTyCell<'a, Cx>, Box<Object>>),
}

impl<'a, Cx> ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Gets the component with given type.
    pub fn get<T: 'static>(&self, ty: &ComponentType<T>) -> Option<&T> {
        match &self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes } => changes
                .get(&RawErasedComponentType::from(ty))
                .map(|o| o.as_ref().and_then(|o| o.downcast_ref::<T>()))
                .unwrap_or_else(|| base.get(ty)),
            MapInner::Simple(map) => map
                .get(&RawErasedComponentType::from(ty))
                .and_then(|any| any.downcast_ref()),
        }
    }

    #[inline]
    fn get_raw<T: 'static>(&self, ty: &ComponentType<T>) -> Option<&Object> {
        match &self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes } => changes
                .get(&RawErasedComponentType::from(ty))
                .map(Option::as_deref)
                .unwrap_or_else(|| base.get_raw(ty)),
            MapInner::Simple(map) => map.get(&RawErasedComponentType::from(ty)).map(|b| &**b),
        }
    }

    /// Gets the component and its type registration with given type.
    pub fn get_key_value<T: 'static>(
        &self,
        ty: &ComponentType<T>,
    ) -> Option<(ErasedComponentType<'a, Cx>, &T)> {
        match &self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes } => changes
                .get_key_value(&RawErasedComponentType::from(ty))
                .map(|(k, o)| {
                    o.as_ref()
                        .and_then(|o| o.downcast_ref::<T>())
                        .map(|o| (k.0, o))
                })
                .unwrap_or_else(|| base.get_key_value(ty)),
            MapInner::Simple(map) => map
                .get_key_value(&RawErasedComponentType::from(ty))
                .and_then(|(k, any)| any.downcast_ref().map(|a| (k.0, a))),
        }
    }

    #[inline]
    fn get_raw_key_value<T: 'static>(
        &self,
        ty: &ComponentType<T>,
    ) -> Option<(ErasedComponentType<'a, Cx>, &Object)> {
        match &self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes } => changes
                .get_key_value(&RawErasedComponentType::from(ty))
                .map(|(a, b)| b.as_deref().map(|b| (a.0, b)))
                .unwrap_or_else(|| base.get_raw_key_value(ty)),
            MapInner::Simple(map) => map
                .get_key_value(&RawErasedComponentType::from(ty))
                .map(|(k, any)| (k.0, &**any)),
        }
    }

    /// Returns whether a component with given type exist.
    pub fn contains<T: 'static>(&self, ty: &ComponentType<T>) -> bool {
        match &self.0 {
            MapInner::Empty => false,
            MapInner::Patched { base, changes } => changes
                .get(&RawErasedComponentType::from(ty))
                .map(|opt| opt.is_some())
                .unwrap_or_else(|| base.contains(ty)),
            MapInner::Simple(map) => map.contains_key(&RawErasedComponentType::from(ty)),
        }
    }

    /// Gets the component with given type, with mutable access.
    pub fn get_mut<T: 'static>(&mut self, ty: &ComponentType<T>) -> Option<&mut T> {
        match &mut self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes } => {
                if !changes.contains_key(&RawErasedComponentType::from(ty)) {
                    let (k, v) = base.get_raw_key_value(ty)?;
                    changes.insert(CompTyCell(k), Some((ty.util.clone)(v)));
                }
                changes
                    .get_mut(&RawErasedComponentType::from(ty))
                    .and_then(Option::as_mut)
                    .and_then(|obj| obj.downcast_mut::<T>())
            }
            MapInner::Simple(map) => map
                .get_mut(&RawErasedComponentType::from(ty))
                .and_then(|any| any.downcast_mut()),
        }
    }

    /// Inserts a component into this map, and returns the old one if valid.
    ///
    /// This function receives a type-erased component type, because it contains the registration information,
    /// which is useful for interacting with Minecraft protocol.
    ///
    /// # Panics
    ///
    /// This function panics when the given component type's type information does not match with
    /// the given static type.
    pub fn insert<T>(&mut self, ty: ErasedComponentType<'a, Cx>, val: T) -> Option<Maybe<'a, T>>
    where
        T: Send + Sync + 'static,
    {
        assert_eq! {
            ty.ty,
            TypeId::of::<T>(),
            "the component type should matches the type of given value",
        };
        match &mut self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes } => {
                let old = base.get_raw(&ty.downcast::<T>()?);
                if old.is_some_and(|old| (ty.util.eq)(&val, old)) {
                    changes.remove(&CompTyCell(ty))
                } else if let Some(v) = changes.insert(CompTyCell(ty), Some(Box::new(val))) {
                    Some(v)
                } else {
                    return old
                        .and_then(|old| old.downcast_ref::<T>())
                        .map(Maybe::Borrowed);
                }
                .flatten()
            }
            MapInner::Simple(map) => map.insert(CompTyCell(ty), Box::new(val)),
        }
        .and_then(|obj| obj.downcast().ok())
        .map(|boxed| Maybe::Owned(SimpleOwned(*boxed)))
    }

    /// Removes a component with given type, and returns it if valid.
    pub fn remove<T: 'static>(&mut self, ty: &ComponentType<T>) -> Option<Maybe<'a, T>> {
        match &mut self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes } => {
                let old = base.get_raw_key_value(ty);
                let now = changes.get_mut(&RawErasedComponentType::from(ty));
                match (old, now) {
                    (Some((k, v)), None) => {
                        changes.insert(CompTyCell(k), None);
                        v.downcast_ref::<T>().map(Maybe::Borrowed)
                    }
                    (_, Some(now)) => now
                        .take()
                        .and_then(|obj| obj.downcast().ok())
                        .map(|boxed| Maybe::Owned(SimpleOwned(*boxed))),
                    (None, None) => None,
                }
            }
            MapInner::Simple(map) => map
                .remove(&RawErasedComponentType::from(ty))
                .and_then(|obj| obj.downcast().ok())
                .map(|boxed| Maybe::Owned(SimpleOwned(*boxed))),
        }
    }
}

impl<Cx: ProvideIdTy> PartialEq for CompTyCell<'_, Cx> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Cx: ProvideIdTy> Eq for CompTyCell<'_, Cx> {}

impl<Cx: ProvideIdTy> Hash for CompTyCell<'_, Cx> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (*self.0).hash(state)
    }
}

impl<Cx: ProvideIdTy> Borrow<RawErasedComponentType<Cx>> for CompTyCell<'_, Cx> {
    #[inline]
    fn borrow(&self) -> &RawErasedComponentType<Cx> {
        &self.0
    }
}
