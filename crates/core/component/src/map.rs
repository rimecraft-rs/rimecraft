//! Component map implementation.

use std::{any::TypeId, borrow::Borrow, collections::hash_map, hash::Hash};

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
        changes_count: isize,
    },
    Simple(AHashMap<CompTyCell<'a, Cx>, Box<Object>>),
}

impl<'a, Cx> ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Gets the component with given type.
    pub fn get<T: 'static>(&self, ty: &ComponentType<T>) -> Option<&T> {
        self.get_raw(&RawErasedComponentType::from(ty))
            .and_then(Object::downcast_ref)
    }

    #[inline]
    fn get_raw(&self, ty: &RawErasedComponentType<Cx>) -> Option<&Object> {
        match &self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes, .. } => changes
                .get(ty)
                .map(Option::as_deref)
                .unwrap_or_else(|| base.get_raw(ty)),
            MapInner::Simple(map) => map.get(ty).map(|b| &**b),
        }
    }

    /// Gets the component and its type registration with given type.
    pub fn get_key_value<T: 'static>(
        &self,
        ty: &ComponentType<T>,
    ) -> Option<(ErasedComponentType<'a, Cx>, &T)> {
        self.get_key_value_raw(&RawErasedComponentType::from(ty))
            .and_then(|(k, v)| v.downcast_ref().map(|v| (k, v)))
    }

    #[inline]
    fn get_key_value_raw(
        &self,
        ty: &RawErasedComponentType<Cx>,
    ) -> Option<(ErasedComponentType<'a, Cx>, &Object)> {
        match &self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes, .. } => changes
                .get_key_value(ty)
                .map(|(a, b)| b.as_deref().map(|b| (a.0, b)))
                .unwrap_or_else(|| base.get_key_value_raw(ty)),
            MapInner::Simple(map) => map.get_key_value(ty).map(|(k, any)| (k.0, &**any)),
        }
    }

    /// Returns whether a component with given type exist.
    pub fn contains<T: 'static>(&self, ty: &ComponentType<T>) -> bool {
        self.contains_raw(&RawErasedComponentType::from(ty))
    }

    #[inline]
    fn contains_raw(&self, ty: &RawErasedComponentType<Cx>) -> bool {
        match &self.0 {
            MapInner::Empty => false,
            MapInner::Patched { base, changes, .. } => changes
                .get(ty)
                .map(|opt| opt.is_some())
                .unwrap_or_else(|| base.contains_raw(ty)),
            MapInner::Simple(map) => map.contains_key(ty),
        }
    }

    /// Gets the component with given type, with mutable access.
    pub fn get_mut<T: 'static>(&mut self, ty: &ComponentType<T>) -> Option<&mut T> {
        self.get_mut_raw(&RawErasedComponentType::from(ty))
            .and_then(Object::downcast_mut)
    }

    #[inline]
    fn get_mut_raw(&mut self, ty: &RawErasedComponentType<Cx>) -> Option<&mut Object> {
        match &mut self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes, .. } => {
                if !changes.contains_key(ty) {
                    let (k, v) = base.get_key_value_raw(ty)?;
                    changes.insert(CompTyCell(k), Some((ty.f.util.clone)(v)));
                }
                changes.get_mut(ty).and_then(Option::as_mut)
            }
            MapInner::Simple(map) => map.get_mut(ty),
        }
        .map(Box::as_mut)
    }

    /// Inserts a component into this map, and returns the old one if valid.
    ///
    /// This function receives a type-erased component type, because it contains the registration
    /// information, which is useful for interacting with Minecraft protocol.
    ///
    /// # Panics
    ///
    /// This function panics when the given component type's type information does not match with
    /// the given static type.
    pub fn insert<T>(&mut self, ty: ErasedComponentType<'a, Cx>, val: T) -> Option<Maybe<'a, T>>
    where
        T: Send + Sync + 'static,
    {
        let value = self.insert_untracked(ty, val);
        if value.is_none() {
            self.track_add()
        }
        value
    }

    #[inline]
    fn insert_untracked<T>(
        &mut self,
        ty: ErasedComponentType<'a, Cx>,
        val: T,
    ) -> Option<Maybe<'a, T>>
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
            MapInner::Patched { base, changes, .. } => {
                let old = base.get_raw(&ty);
                if old.is_some_and(|old| (ty.f.util.eq)(&val, old)) {
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
        let value = self.remove_untracked(ty);
        if value.is_some() {
            self.track_rm()
        }
        value
    }

    #[inline]
    fn remove_untracked<T: 'static>(&mut self, ty: &ComponentType<T>) -> Option<Maybe<'a, T>> {
        match &mut self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes, .. } => {
                let era_ty = &RawErasedComponentType::from(ty);
                let old = base.get_key_value_raw(era_ty);
                let now = changes.get_mut(era_ty);
                match (old, now) {
                    (Some((k, v)), None) => {
                        changes.insert(CompTyCell(k), None);
                        v.downcast_ref::<T>().map(Maybe::Borrowed)
                    }
                    (Some(_), Some(now)) => now
                        .take()
                        .and_then(|obj| obj.downcast().ok())
                        .map(|boxed| Maybe::Owned(SimpleOwned(*boxed))),
                    (None, Some(_)) => changes
                        .remove(era_ty)?
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

    #[inline]
    fn track_add(&mut self) {
        if let MapInner::Patched { changes_count, .. } = &mut self.0 {
            *changes_count += 1;
        }
    }

    #[inline]
    fn track_rm(&mut self) {
        if let MapInner::Patched { changes_count, .. } = &mut self.0 {
            *changes_count -= 1;
        }
    }

    /// Returns the count of valid components.
    pub fn len(&self) -> usize {
        self._len()
    }

    #[inline(always)]
    fn _len(&self) -> usize {
        match &self.0 {
            MapInner::Empty => 0,
            MapInner::Patched {
                base,
                changes_count,
                ..
            } => ((base.len() as isize) + changes_count) as usize,
            MapInner::Simple(map) => map.len(),
        }
    }

    /// Returns whether this map is empty.
    pub fn is_empty(&self) -> bool {
        self._len() == 0
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

pub struct Iter<'a, Cx>(IterInner<'a, Cx>)
where
    Cx: ProvideIdTy;

enum IterInner<'a, Cx: ProvideIdTy> {
    Empty,
    Patched {
        changes: &'a AHashMap<CompTyCell<'a, Cx>, Option<Box<Object>>>,
        base_it: Box<Iter<'a, Cx>>,
        changes_it: hash_map::Iter<'a, CompTyCell<'a, Cx>, Option<Box<Object>>>,
    },
    Simple(hash_map::Iter<'a, CompTyCell<'a, Cx>, Box<Object>>),
}

impl<'a, Cx> Iterator for Iter<'a, Cx>
where
    Cx: ProvideIdTy,
{
    type Item = (ErasedComponentType<'a, Cx>, &'a Object);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            IterInner::Empty => None,
            IterInner::Patched {
                changes,
                base_it,
                changes_it,
            } => {
                for (k, v) in changes_it {
                    if let Some(v) = v {
                        return Some((k.0, &**v));
                    }
                }

                for (k, v) in base_it {
                    let patched = changes.get(&CompTyCell(k));
                    match patched {
                        Some(Some(opt)) => {
                            return Some((k, &**opt));
                        }
                        Some(None) => continue,
                        None => return Some((k, v)),
                    }
                }

                None
            }
            IterInner::Simple(it) => it.next().map(|(k, v)| (k.0, &**v)),
        }
    }
}
