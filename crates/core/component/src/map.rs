//! Component map implementation.

use std::{
    any::TypeId, borrow::Borrow, collections::hash_map, fmt::Debug, hash::Hash, marker::PhantomData,
};

use ahash::AHashMap;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_maybe::{Maybe, SimpleOwned};
use rimecraft_registry::ProvideRegistry;
use serde::{Deserialize, Serialize};

use crate::{
    changes::ComponentChanges, ComponentType, ErasedComponentType, Object, RawErasedComponentType,
    SerdeCodec,
};

#[repr(transparent)]
pub(crate) struct CompTyCell<'a, Cx: ProvideIdTy>(ErasedComponentType<'a, Cx>);

/// A map that stores components.
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

impl<Cx> Default for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy,
{
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, Cx> ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Creates an empty component map.
    #[inline]
    pub const fn empty() -> Self {
        Self(MapInner::Empty)
    }

    /// Creates a **patched** component map with given base map.
    #[inline]
    pub fn new(base: &'a ComponentMap<'a, Cx>) -> Self {
        Self(MapInner::Patched {
            base,
            changes: AHashMap::new(),
            changes_count: 0,
        })
    }

    /// Returns a builder for creating a simple component map.
    #[inline]
    pub fn builder() -> Builder<'a, Cx> {
        Builder {
            map: AHashMap::new(),
        }
    }

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

    /// Returns an iterator over the components in this map.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Cx> {
        self.into_iter()
    }

    /// Returns the changes of this map.
    pub fn changes(&self) -> Option<ComponentChanges<'a, '_, Cx>> {
        if let MapInner::Patched { changes, .. } = &self.0 {
            Some(ComponentChanges {
                changes: Maybe::Borrowed(changes),
            })
        } else {
            None
        }
    }
}

impl<'a, Cx> IntoIterator for &'a ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy,
{
    type Item = <Iter<'a, Cx> as Iterator>::Item;

    type IntoIter = Iter<'a, Cx>;

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

/// Iterates over the components in this map.
pub struct Iter<'a, Cx>(IterInner<'a, Cx>, &'a ComponentMap<'a, Cx>)
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.1.len();
        (len, Some(len))
    }
}

/// A builder for creating a simple component map.
pub struct Builder<'a, Cx>
where
    Cx: ProvideIdTy,
{
    map: AHashMap<CompTyCell<'a, Cx>, Box<Object>>,
}

impl<'a, Cx> Builder<'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Inserts a component into this map.
    ///
    /// # Panics
    ///
    /// This function panics when the given component type's type information does not match with
    /// the given static type.
    #[inline]
    pub fn insert<T>(mut self, ty: ErasedComponentType<'a, Cx>, val: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        assert_eq!(
            ty.ty,
            TypeId::of::<T>(),
            "the component type should matches the type of given value"
        );
        self.map.insert(CompTyCell(ty), Box::new(val));
        self
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

impl<'a, Cx> From<Builder<'a, Cx>> for ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy,
{
    #[inline]
    fn from(builder: Builder<'a, Cx>) -> Self {
        builder.build()
    }
}

impl<Cx> Debug for CompTyCell<'_, Cx>
where
    Cx: ProvideIdTy + Debug,
    Cx::Id: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<Cx> Debug for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy + Debug,
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
                .field("changes", changes)
                .field("changes_count", changes_count)
                .finish(),
            MapInner::Simple(map) => f.debug_tuple("SimpleComponentMap").field(&map).finish(),
        }
    }
}

impl<Cx> Debug for Iter<'_, Cx>
where
    Cx: ProvideIdTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            IterInner::Empty => f.write_str("EmptyComponentMapIter"),
            IterInner::Patched {
                changes,
                base_it,
                changes_it,
            } => f
                .debug_struct("PatchedComponentMapIter")
                .field("changes", changes)
                .field("base_it", base_it)
                .field("changes_it", changes_it)
                .finish(),
            IterInner::Simple(it) => f.debug_tuple("SimpleComponentMapIter").field(it).finish(),
        }
    }
}

impl<Cx> Debug for Builder<'_, Cx>
where
    Cx: ProvideIdTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentMapBuilder")
            .field("map", &self.map)
            .finish()
    }
}

impl<Cx> Serialize for ComponentMap<'_, Cx>
where
    Cx: ProvideIdTy,
    Cx::Id: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        for (ty, val) in self {
            if let Some(codec) = &ty.f.serde_codec {
                map.serialize_entry(&ty, (codec.ser)(val))?;
            }
        }
        map.end()
    }
}

impl<'a, 'de, Cx> Deserialize<'de> for ComponentMap<'a, Cx>
where
    Cx: ProvideIdTy + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<Cx>>,
    Cx::Id: Deserialize<'de> + Hash + Eq,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a, Cx>(PhantomData<&'a Cx>);

        impl<'a, 'de, Cx> serde::de::Visitor<'de> for Visitor<'a, Cx>
        where
            Cx: ProvideIdTy + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<Cx>>,
            Cx::Id: Deserialize<'de> + Hash + Eq,
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
                struct DeSeed<'a, Cx>(&'a SerdeCodec, PhantomData<&'a Cx>);

                impl<'a, 'de, Cx> serde::de::DeserializeSeed<'de> for DeSeed<'a, Cx>
                where
                    Cx: ProvideIdTy + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<Cx>>,
                    Cx::Id: Deserialize<'de> + Hash + Eq,
                {
                    type Value = Box<Object>;

                    #[inline]
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        (self.0.de)(&mut <dyn erased_serde::Deserializer<'de>>::erase(
                            deserializer,
                        ))
                        .map_err(serde::de::Error::custom)
                    }
                }
                while let Some(k) = map.next_key::<ErasedComponentType<'a, Cx>>()? {
                    let codec = k.f.serde_codec.as_ref().ok_or_else(|| {
                        serde::de::Error::invalid_type(
                            serde::de::Unexpected::Other("transient component type"),
                            &"persistent component type",
                        )
                    })?;
                    m.insert(
                        CompTyCell(k),
                        map.next_value_seed(DeSeed(codec, PhantomData::<&Cx>))?,
                    );
                }
                m.shrink_to_fit();
                Ok(Builder { map: m }.build())
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a component map")
            }
        }

        deserializer.deserialize_map(Visitor(PhantomData))
    }
}
