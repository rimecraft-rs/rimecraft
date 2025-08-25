//! Component map implementation.

use std::{
    borrow::Borrow, cell::UnsafeCell, collections::hash_map, fmt::Debug, hash::Hash,
    marker::PhantomData, sync::Arc,
};

use ahash::AHashMap;
use local_cx::{
    LocalContext, LocalContextExt as _, ProvideLocalCxTy, WithLocalCx,
    dyn_codecs::{self, Any},
    dyn_cx::AsDynamicContext,
    serde::{DeserializeWithCx, SerializeWithCx},
};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_maybe::{Maybe, SimpleOwned};
use rimecraft_registry::Registry;
use serde::{Deserialize, Serialize};

use crate::{
    ComponentType, ErasedComponentType, Object, RawErasedComponentType, UnsafeDebugIter,
    UnsafeSerdeCodec, changes::ComponentChanges,
};

#[repr(transparent)]
pub(crate) struct CompTyCell<'a, Cx>(pub(crate) ErasedComponentType<'a, Cx>)
where
    Cx: ProvideIdTy + ProvideLocalCxTy;

/// A map that stores components.
pub struct ComponentMap<'a, Cx>(MapInner<'a, Cx>)
where
    Cx: ProvideIdTy + ProvideLocalCxTy;

enum MapInner<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    Empty,
    Patched {
        base: Maybe<'a, ComponentMap<'a, Cx>, Arc<ComponentMap<'a, Cx>>>,
        changes: AHashMap<CompTyCell<'a, Cx>, Option<Box<Object<'a>>>>,
        changes_count: isize,
    },
    Simple(AHashMap<CompTyCell<'a, Cx>, Box<Object<'a>>>),
}

impl<Cx> Default for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<'a, Cx> ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    /// An empty component map.
    pub const EMPTY: Self = Self(MapInner::Empty);

    /// Creates an empty component map.
    #[deprecated = "use `ComponentMap::EMPTY` instead"]
    #[inline]
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    /// Creates a **patched** component map with given base map.
    #[inline]
    pub fn new(base: &'a ComponentMap<'a, Cx>) -> Self {
        Self(MapInner::Patched {
            base: Maybe::Borrowed(base),
            changes: AHashMap::new(),
            changes_count: 0,
        })
    }

    /// Creates a **patched** component map with given base map.
    #[inline]
    pub fn arc_new(base: Arc<ComponentMap<'a, Cx>>) -> Self {
        Self(MapInner::Patched {
            base: Maybe::Owned(base),
            changes: AHashMap::new(),
            changes_count: 0,
        })
    }

    /// Creates a **patched** component map with given base map and changes.
    #[inline]
    pub fn with_changes(
        base: &'a ComponentMap<'a, Cx>,
        changes: ComponentChanges<'a, '_, Cx>,
    ) -> Self {
        Self::with_changes_raw(Maybe::Borrowed(base), changes)
    }

    /// Creates a **patched** component map with given base map and changes.
    #[inline]
    pub fn arc_with_changes(
        base: Arc<ComponentMap<'a, Cx>>,
        changes: ComponentChanges<'a, '_, Cx>,
    ) -> Self {
        Self::with_changes_raw(Maybe::Owned(base), changes)
    }

    fn with_changes_raw(
        base: Maybe<'a, ComponentMap<'a, Cx>, Arc<ComponentMap<'a, Cx>>>,
        changes: ComponentChanges<'a, '_, Cx>,
    ) -> Self {
        Self(MapInner::Patched {
            changes_count: changes
                .changed
                .iter()
                .map(|(&CompTyCell(k), v)| {
                    let occupied = base.contains_raw(&k);
                    if v.is_some() {
                        if occupied { 0 } else { 1 }
                    } else if occupied {
                        -1
                    } else {
                        0
                    }
                })
                .sum(),
            base,
            changes: match changes.changed {
                Maybe::Borrowed(c) => c
                    .iter()
                    .map(|(k, v)| (CompTyCell(k.0), v.as_deref().map(k.0.f.util.clone)))
                    .collect(),
                Maybe::Owned(c) => c.0,
            },
        })
    }

    /// Returns a builder for creating a simple component map.
    #[inline]
    pub fn builder() -> Builder<'a, Cx> {
        Builder {
            map: AHashMap::new(),
        }
    }

    /// Returns a builder for creating a simple component map with given capacity.
    #[inline]
    pub fn builder_with_capacity(capacity: usize) -> Builder<'a, Cx> {
        Builder {
            map: AHashMap::with_capacity(capacity),
        }
    }

    /// Gets the component with given type.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn get<T>(&self, ty: &ComponentType<'a, T, Cx>) -> Option<&T> {
        self.get_raw(&RawErasedComponentType::from(ty))
            .and_then(|val| unsafe { <dyn Any>::downcast_ref(val) })
    }

    /// Gets the component with given type.
    ///
    /// This function is similar to `get`, but it returns the raw object instead of the reference.
    pub fn get_raw(&self, ty: &RawErasedComponentType<'a, Cx>) -> Option<&Object<'a>> {
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
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn get_key_value<T>(
        &self,
        ty: &ComponentType<'a, T, Cx>,
    ) -> Option<(ErasedComponentType<'a, Cx>, &T)> {
        unsafe {
            self.get_key_value_raw(&RawErasedComponentType::from(ty))
                .and_then(|(k, v)| <dyn Any>::downcast_ref(v).map(|v| (k, v)))
        }
    }

    /// Gets the component and its type registration with given type.
    ///
    /// This function is similar to `get_key_value`, but it returns the raw object instead of the reference.
    pub fn get_key_value_raw(
        &self,
        ty: &RawErasedComponentType<'a, Cx>,
    ) -> Option<(ErasedComponentType<'a, Cx>, &Object<'a>)> {
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
    pub fn contains<T>(&self, ty: &ComponentType<'a, T, Cx>) -> bool {
        self.contains_raw(&RawErasedComponentType::from(ty))
    }

    /// Returns whether a component with given type exist.
    ///
    /// This function is similar to `contains`, but it receives the raw component type instead of the typed one.
    pub fn contains_raw(&self, ty: &RawErasedComponentType<'a, Cx>) -> bool {
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
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn get_mut<T>(&mut self, ty: &ComponentType<'a, T, Cx>) -> Option<&mut T> {
        self.get_mut_raw(&RawErasedComponentType::from(ty))
            .and_then(|val| unsafe { <dyn Any>::downcast_mut(val) })
    }

    /// Gets the component with given type, with mutable access.
    ///
    /// This function is similar to `get_mut`, but it returns the raw object instead of the reference.
    pub fn get_mut_raw(&mut self, ty: &RawErasedComponentType<'a, Cx>) -> Option<&mut Object<'a>> {
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
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn insert<T>(
        &mut self,
        ty: ErasedComponentType<'a, Cx>,
        val: T,
    ) -> Option<Maybe<'_, T>>
    where
        T: Send + Sync + 'a,
    {
        let ptr = self as *mut Self;
        let value = unsafe { self.insert_untracked(ty, val) };
        if value.is_none() {
            //SAFETY: this does not affect the lifetime of the value.
            unsafe { (*ptr).track_add() }
        }
        value
    }

    #[inline]
    unsafe fn insert_untracked<T>(
        &mut self,
        ty: ErasedComponentType<'a, Cx>,
        val: T,
    ) -> Option<Maybe<'_, T>>
    where
        T: Send + Sync + 'a,
    {
        assert_eq! {
            ty.ty,
            typeid::of::<T>(),
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
                        .and_then(|old| unsafe { <dyn Any>::downcast_ref::<T>(old) })
                        .map(Maybe::Borrowed);
                }
                .flatten()
            }
            MapInner::Simple(map) => map.insert(CompTyCell(ty), Box::new(val)),
        }
        .and_then(|obj| unsafe { dyn_codecs::downcast_boxed(obj).ok() })
        .map(|boxed| Maybe::Owned(SimpleOwned(*boxed)))
    }

    /// Removes a component with given type, and returns it if valid.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    pub unsafe fn remove<T>(&mut self, ty: &ComponentType<'a, T, Cx>) -> Option<Maybe<'_, T>> {
        let ptr = self as *mut Self;
        let value = unsafe { self.remove_untracked(ty) };
        if value.is_some() {
            //SAFETY: this does not affect the lifetime of the value.
            unsafe { (*ptr).track_rm() }
        }
        value
    }

    #[inline]
    unsafe fn remove_untracked<T>(
        &mut self,
        ty: &ComponentType<'a, T, Cx>,
    ) -> Option<Maybe<'_, T>> {
        match &mut self.0 {
            MapInner::Empty => None,
            MapInner::Patched { base, changes, .. } => {
                let era_ty = &RawErasedComponentType::from(ty);
                let old = base.get_key_value_raw(era_ty);
                let now = changes.get_mut(era_ty);
                match (old, now) {
                    (Some((k, v)), None) => {
                        changes.insert(CompTyCell(k), None);
                        let v: &(dyn Any + '_) = v;
                        unsafe { v.downcast_ref::<T>().map(Maybe::Borrowed) }
                    }
                    (Some(_), Some(now)) => now
                        .take()
                        .and_then(|obj| unsafe { dyn_codecs::downcast_boxed(obj).ok() })
                        .map(|boxed| Maybe::Owned(SimpleOwned(*boxed))),
                    (None, Some(_)) => changes
                        .remove(era_ty)?
                        .and_then(|obj| unsafe { dyn_codecs::downcast_boxed(obj).ok() })
                        .map(|boxed| Maybe::Owned(SimpleOwned(*boxed))),
                    (None, None) => None,
                }
            }
            MapInner::Simple(map) => map
                .remove(&RawErasedComponentType::from(ty))
                .and_then(|obj| unsafe { dyn_codecs::downcast_boxed(obj).ok() })
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

    /// Returns an iterator over the components in this map.
    #[inline]
    pub fn iter<'s>(&'s self) -> Iter<'s, 'a, Cx> {
        self.into_iter()
    }

    /// Returns the changes of this map.
    pub fn changes(&self) -> Option<ComponentChanges<'a, '_, Cx>> {
        if let MapInner::Patched { changes, .. } = &self.0 {
            Some(ComponentChanges {
                changed: Maybe::Borrowed(changes),
                ser_count: changes.keys().filter(|cell| !cell.0.is_transient()).count(),
            })
        } else {
            None
        }
    }
}

impl<'a, 's, Cx> IntoIterator for &'s ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    type Item = <Iter<'s, 'a, Cx> as Iterator>::Item;

    type IntoIter = Iter<'s, 'a, Cx>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(
            match &self.0 {
                MapInner::Empty => IterInner::Empty,
                MapInner::Patched { base, changes, .. } => IterInner::Patched {
                    base_it: Box::new(base.into_iter()),
                    changes,
                    changes_it: changes.iter(),
                },
                MapInner::Simple(map) => IterInner::Simple(map.iter()),
            },
            self,
        )
    }
}

impl<Cx: ProvideIdTy + ProvideLocalCxTy> PartialEq for CompTyCell<'_, Cx> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl<Cx: ProvideIdTy + ProvideLocalCxTy> Eq for CompTyCell<'_, Cx> {}

impl<Cx: ProvideIdTy + ProvideLocalCxTy> Hash for CompTyCell<'_, Cx> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (*self.0).hash(state)
    }
}

impl<'a, Cx: ProvideIdTy + ProvideLocalCxTy> Borrow<RawErasedComponentType<'a, Cx>>
    for CompTyCell<'a, Cx>
{
    #[inline]
    fn borrow(&self) -> &RawErasedComponentType<'a, Cx> {
        &self.0
    }
}

/// Iterates over the components in this map.
pub struct Iter<'s, 'a, Cx>(IterInner<'s, 'a, Cx>, &'s ComponentMap<'a, Cx>)
where
    Cx: ProvideIdTy + ProvideLocalCxTy;

enum IterInner<'s, 'a, Cx: ProvideIdTy + ProvideLocalCxTy> {
    Empty,
    Patched {
        changes: &'s AHashMap<CompTyCell<'a, Cx>, Option<Box<Object<'a>>>>,
        base_it: Box<Iter<'s, 'a, Cx>>,
        changes_it: hash_map::Iter<'s, CompTyCell<'a, Cx>, Option<Box<Object<'a>>>>,
    },
    Simple(hash_map::Iter<'s, CompTyCell<'a, Cx>, Box<Object<'a>>>),
}

impl<'s, 'a, Cx> Iterator for Iter<'s, 'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    type Item = (ErasedComponentType<'a, Cx>, &'s Object<'a>);

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
                        Some(_) => continue,
                        None => return Some((k, v)),
                    }
                }

                None
            }
            IterInner::Simple(it) => it.next().map(|(k, v)| (k.0, &**v)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.1.len();
        (len, Some(len))
    }
}

impl<Cx> ExactSizeIterator for Iter<'_, '_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    #[inline]
    fn len(&self) -> usize {
        self.1.len()
    }
}

impl<Cx> PartialEq for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(move |(ty, obj)| other.get_raw(&*ty).is_some_and(|o| (ty.f.util.eq)(obj, o)))
    }
}

impl<Cx> Eq for ComponentMap<'_, Cx> where Cx: ProvideIdTy + ProvideLocalCxTy {}

impl<Cx> Hash for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (ty, obj) in self.iter() {
            ty.hash(state);
            (ty.f.util.hash)(obj, state);
        }
    }
}

impl<Cx> Clone for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    fn clone(&self) -> Self {
        match &self.0 {
            MapInner::Empty => Self::EMPTY,
            MapInner::Patched {
                base,
                changes,
                changes_count,
            } => Self(MapInner::Patched {
                base: Maybe::clone(base),
                changes: changes
                    .iter()
                    .map(|(k, v)| (CompTyCell(k.0), v.as_deref().map(k.0.f.util.clone)))
                    .collect(),
                changes_count: *changes_count,
            }),
            MapInner::Simple(map) => Self(MapInner::Simple(
                map.iter()
                    .map(|(k, v)| (CompTyCell(k.0), (k.0.f.util.clone)(&**v)))
                    .collect(),
            )),
        }
    }
}

/// A builder for creating a simple component map.
pub struct Builder<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    map: AHashMap<CompTyCell<'a, Cx>, Box<Object<'a>>>,
}

impl<'a, Cx> Builder<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    /// Inserts a component into this map.
    ///
    /// # Panics
    ///
    /// This function panics when the given component type's type information does not match with
    /// the given static type.
    pub fn insert<T>(&mut self, ty: ErasedComponentType<'a, Cx>, val: T)
    where
        T: Send + Sync + 'a,
    {
        assert_eq!(
            ty.ty,
            typeid::of::<T>(),
            "the component type should matches the type of given value"
        );
        self.map.insert(CompTyCell(ty), Box::new(val));
    }

    /// Inserts a component into this map.
    ///
    /// This function is similar to `insert`, but it receives the raw component type instead of the typed one.
    ///
    /// # Panics
    ///
    /// This function panics when the given component type's type information does not match with
    /// the type of given value.
    #[inline]
    pub fn insert_raw(&mut self, ty: ErasedComponentType<'a, Cx>, val: Box<Object<'a>>) {
        assert_eq!(
            ty.ty,
            (*val).type_id(),
            "the component type should matches the type of given value"
        );
        self.map.insert(CompTyCell(ty), val);
    }

    /// Builds the component map.
    pub fn build(self) -> ComponentMap<'a, Cx> {
        if self.map.is_empty() {
            ComponentMap(MapInner::Empty)
        } else {
            ComponentMap(MapInner::Simple(self.map))
        }
    }
}

impl<'a, Cx> Extend<(ErasedComponentType<'a, Cx>, Box<Object<'a>>)> for Builder<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    fn extend<T: IntoIterator<Item = (ErasedComponentType<'a, Cx>, Box<Object<'a>>)>>(
        &mut self,
        iter: T,
    ) {
        self.map.extend(
            iter.into_iter()
                .filter(|(k, v)| k.ty == (**v).type_id())
                .map(|(k, v)| (CompTyCell(k), v)),
        );
    }
}

impl<'a, 's, Cx> Extend<(ErasedComponentType<'a, Cx>, &'s Object<'a>)> for Builder<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    #[inline]
    fn extend<T: IntoIterator<Item = (ErasedComponentType<'a, Cx>, &'s Object<'a>)>>(
        &mut self,
        iter: T,
    ) {
        self.extend(iter.into_iter().map(|(k, v)| (k, (k.f.util.clone)(v))));
    }
}

impl<'a, Cx> From<Builder<'a, Cx>> for ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
{
    #[inline]
    fn from(builder: Builder<'a, Cx>) -> Self {
        builder.build()
    }
}

impl<Cx> Debug for CompTyCell<'_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy + Debug,
    Cx::Id: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<Cx> Debug for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            MapInner::Empty => f.write_str("EmptyComponentMap"),
            MapInner::Patched {
                base,
                changes,
                changes_count,
            } => f
                .debug_struct("PatchedComponentMap")
                .field("base", base)
                .field(
                    "changes",
                    &UnsafeDebugIter(UnsafeCell::new(
                        changes
                            .iter()
                            .map(|(k, v)| (k, v.as_ref().map(|obj| (k.0.f.util.dbg)(obj)))),
                    )),
                )
                .field("changes_count", changes_count)
                .finish(),
            MapInner::Simple(map) => f
                .debug_tuple("SimpleComponentMap")
                .field(&UnsafeDebugIter(UnsafeCell::new(
                    map.iter().map(|(k, v)| (k, (k.0.f.util.dbg)(v))),
                )))
                .finish(),
        }
    }
}

impl<Cx> Debug for Iter<'_, '_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            IterInner::Empty => f.write_str("EmptyComponentMapIter"),
            IterInner::Patched {
                changes,
                base_it,
                changes_it: _,
            } => f
                .debug_struct("PatchedComponentMapIter")
                .field("changes", &UnsafeDebugIter(UnsafeCell::new(changes.keys())))
                .field("base_it", base_it)
                .finish(),
            IterInner::Simple(_it) => f.debug_tuple("SimpleComponentMapIter").finish(),
        }
    }
}

impl<Cx> Debug for Builder<'_, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentMapBuilder")
            .field("map", &UnsafeDebugIter(UnsafeCell::new(self.map.keys())))
            .finish()
    }
}

impl<'a, Cx> SerializeWithCx<Cx::LocalContext<'a>> for ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
    Cx::Id: Serialize,
{
    fn serialize_with_cx<S>(
        &self,
        serializer: WithLocalCx<S, Cx::LocalContext<'a>>,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let cx = serializer.local_cx;
        let mut map = serializer.inner.serialize_map(None)?;

        for (ty, val) in self.iter() {
            if let Some(codec) = ty.f.serde_codec {
                map.serialize_entry(&ty, (codec.ser)(&cx.with(val)))?;
            }
        }
        map.end()
    }
}

impl<'a, Cx> SerializeWithCx<Cx::LocalContext<'a>> for &ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
    Cx::Id: Serialize,
{
    #[inline]
    fn serialize_with_cx<S>(
        &self,
        serializer: WithLocalCx<S, Cx::LocalContext<'a>>,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (**self).serialize_with_cx(serializer)
    }
}

impl<'a, 'de, Cx> DeserializeWithCx<'de, Cx::LocalContext<'a>> for ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy + ProvideLocalCxTy,
    Cx::Id: Deserialize<'de> + Hash + Eq,
    Cx::LocalContext<'a>:
        LocalContext<&'a Registry<Cx::Id, RawErasedComponentType<'a, Cx>>> + AsDynamicContext,
{
    fn deserialize_with_cx<D>(
        deserializer: WithLocalCx<D, Cx::LocalContext<'a>>,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a, Cx>(PhantomData<&'a Cx>, Cx::LocalContext<'a>)
        where
            Cx: ProvideLocalCxTy;

        impl<'a, 'de, Cx> serde::de::Visitor<'de> for Visitor<'a, Cx>
        where
            Cx: ProvideIdTy + ProvideLocalCxTy,
            Cx::Id: DeserializeWithCx<'de, Cx::LocalContext<'a>> + Hash + Eq,
            Cx::LocalContext<'a>: LocalContext<&'a Registry<Cx::Id, RawErasedComponentType<'a, Cx>>>
                + AsDynamicContext,
        {
            type Value = ComponentMap<'a, Cx>;

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut m = if let Some(sz) = map.size_hint() {
                    AHashMap::with_capacity(sz)
                } else {
                    AHashMap::new()
                };
                struct DeSeed<'a, 's, Cx: ProvideLocalCxTy>(
                    &'s UnsafeSerdeCodec<'a, Cx::LocalContext<'a>>,
                    PhantomData<Cx>,
                    Cx::LocalContext<'a>,
                );

                impl<'a, 'de, Cx: ProvideLocalCxTy> serde::de::DeserializeSeed<'de> for DeSeed<'a, '_, Cx> {
                    type Value = Box<Object<'a>>;

                    #[inline]
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        (self.0.de)(
                            &mut <dyn erased_serde::Deserializer<'de>>::erase(deserializer),
                            self.2,
                        )
                        .map_err(serde::de::Error::custom)
                    }
                }

                while let Some(k) = map.next_key_seed(WithLocalCx {
                    inner: PhantomData::<ErasedComponentType<'a, Cx>>,
                    local_cx: self.1,
                })? {
                    let codec = k.f.serde_codec.as_ref().ok_or_else(|| {
                        serde::de::Error::invalid_type(
                            serde::de::Unexpected::Other("transient component type"),
                            &"persistent component type",
                        )
                    })?;
                    m.insert(
                        CompTyCell(k),
                        map.next_value_seed(DeSeed(codec, PhantomData::<Cx>, self.1))?,
                    );
                }
                m.shrink_to_fit();
                Ok(Builder { map: m }.build())
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a component map")
            }
        }

        let cx = deserializer.local_cx;
        deserializer.inner.deserialize_map(Visitor(PhantomData, cx))
    }
}
