//! General-purpose descriptor set system for Rust.

use std::{fmt::Debug, marker::PhantomData};

use ident_hash::{HashTableExt as _, IHashMap};

#[cfg(any(feature = "simple-registry", test))]
mod simple_registry;

#[cfg(feature = "simple-registry")]
pub use simple_registry::SimpleRegistry;

pub mod primitives;

/// The maximum number of elements in a slice or vector before it switches to a map.
const SLICE_THRESHOLD: usize = 8;

/// A type of descriptor that is registered into some registry.
///
/// Type `T` must share the same memory layout as `*const ()`.
#[derive(Debug, Clone, Copy)]
pub struct Type<T> {
    registry_marker: *const (),
    index: usize,
    // make T invariant here for safety considerations
    _marker: PhantomData<*mut T>,
}

impl<T> Type<T> {
    const __TYPE_CHECK: () = {
        let layout_t = std::alloc::Layout::new::<T>();
        let layout_ptr = std::alloc::Layout::new::<*const ()>();
        assert!(
            layout_t.size() == layout_ptr.size(),
            "DescriptorType must be the same size as a pointer"
        );
        assert!(
            layout_t.align() == layout_ptr.align(),
            "DescriptorType must be the same alignment as a pointer"
        )
    };

    /// Creates a new descriptor type by registering it into the given registry if it hasn't been
    /// registered yet.
    ///
    /// # Panics
    ///
    /// Panics if the type `T`'s size is not sharing the same memory layout as a thin pointer.
    #[inline]
    pub fn new<R>(registry: &mut R, id: R::Identifier) -> Option<Self>
    where
        R: Registry,
    {
        let _: () = Self::__TYPE_CHECK;
        registry.register::<T>(id).map(|id| Self {
            registry_marker: registry.marker(),
            index: id,
            _marker: PhantomData,
        })
    }
}

/// A set of descriptors.
///
/// A descriptor set holds an unique marker of its registry, avoiding the possibility
/// of accessed by a descriptor from exotic registries.
///
/// Descriptor sets support inheritance. See [`Self::builder_with_parent`].
/// The intended behavior is that the current sets override the parent's descriptors.
#[derive(Debug, Clone)]
pub struct DescriptorSet<'a, 'p> {
    inner: DescriptorSetInner,
    registry_marker: *const (),
    parent: Option<&'p DescriptorSet<'a, 'p>>,
    // role of lifetime: constrain this type not to outlive it.
    _marker: PhantomData<&'a ()>,
}

/// Builder of a [`DescriptorSet`].
#[derive(Debug, Clone)]
pub struct DescriptorSetBuilder<'a, 'p> {
    map: IHashMap<usize, *const ()>,
    registry_marker: *const (),
    max_index: usize,
    parent: Option<&'p DescriptorSet<'a, 'p>>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> DescriptorSet<'a, '_> {
    /// Returns builder of this type.
    #[inline]
    pub fn builder<'p>() -> DescriptorSetBuilder<'a, 'p> {
        DescriptorSetBuilder::new()
    }

    /// Returns builder of this type with a parent.
    #[inline]
    pub fn builder_with_parent<'p>(parent: &'p Self) -> DescriptorSetBuilder<'a, 'p> {
        DescriptorSetBuilder::with_parent(parent)
    }

    /// Returns the value of the given type if present.
    pub fn get<T: Copy>(&self, ty: Type<T>) -> Option<T> {
        if self.registry_marker != ty.registry_marker {
            return None;
        }
        self.__get_unchecked(ty)
    }

    fn __get_unchecked<T: Copy>(&self, ty: Type<T>) -> Option<T> {
        let _: () = Type::<T>::__TYPE_CHECK;
        let raw = self.inner.get(ty.index);
        (!raw.is_null())
            .then(|| {
                let borrowed = &raw;
                let ptr = std::ptr::from_ref(borrowed) as *const T;
                unsafe { *ptr }
            })
            .or_else(|| self.parent.and_then(|p| p.__get_unchecked(ty)))
    }

    /// Checks whether the given type is present.
    pub fn contains<T>(&self, ty: Type<T>) -> bool {
        if self.registry_marker != ty.registry_marker {
            return false;
        }
        self.__contains_unchecked(ty)
    }

    fn __contains_unchecked<T>(&self, ty: Type<T>) -> bool {
        let _: () = Type::<T>::__TYPE_CHECK;
        self.inner.contains(ty.index)
            || self
                .parent
                .map(|p| p.__contains_unchecked(ty))
                .unwrap_or(false)
    }

    /// Returns whether this set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty() && self.parent.map(|p| p.is_empty()).unwrap_or(true)
    }
}

impl<'a, 'p> DescriptorSet<'a, 'p> {
    fn to_builder(&self) -> DescriptorSetBuilder<'a, 'p> {
        DescriptorSetBuilder {
            map: match &self.inner {
                DescriptorSetInner::Slice(items) => items
                    .iter()
                    .enumerate()
                    .filter_map(|(k, &v)| if v.is_null() { None } else { Some((k, v)) })
                    .collect(),
                DescriptorSetInner::Map(hash_map) => hash_map.clone(),
                DescriptorSetInner::Empty => IHashMap::new(),
            },
            registry_marker: self.registry_marker,
            parent: self.parent,
            _marker: PhantomData,

            // no business inside use cases
            max_index: 0,
        }
    }
}

impl<'a, 'p> DescriptorSetBuilder<'a, 'p> {
    /// Creates a new descriptor set builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: IHashMap::new(),
            registry_marker: std::ptr::null(),
            max_index: usize::MIN,
            parent: None,
            _marker: PhantomData,
        }
    }

    /// Creates a new descriptor set builder with a parent.
    #[inline]
    pub fn with_parent(parent: &'p DescriptorSet<'a, 'p>) -> Self {
        Self {
            map: IHashMap::new(),
            registry_marker: parent.registry_marker,
            max_index: usize::MIN,
            parent: Some(parent),
            _marker: PhantomData,
        }
    }

    /// Inserts a value into the descriptor set, returns if the insertion was successful.
    ///
    /// # Panics
    ///
    /// Panics if the type is not under same registry as the builder.
    pub fn insert<T>(&mut self, ty: Type<T>, value: T) -> bool
    where
        T: Copy + Send + Sync + 'a,
    {
        let _: () = Type::<T>::__TYPE_CHECK;

        if self.map.is_empty() {
            self.registry_marker = ty.registry_marker;
        } else {
            assert_eq!(
                self.registry_marker, ty.registry_marker,
                "not under the same registry"
            );
        }

        if self.max_index < ty.index {
            self.max_index = ty.index;
        }

        if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry(ty.index) {
            let borrowed = &value;
            let ptr = std::ptr::from_ref(borrowed) as *const *const ();
            e.insert(unsafe { *ptr });
            true
        } else {
            false
        }
    }

    /// Flattens the builder, merging parent into children.
    ///
    /// This is useful for approaching the best performance of a descriptor set.
    pub fn flatten(self) -> DescriptorSetBuilder<'a, 'static> {
        if let Some(parent) = self.parent {
            let mut map = parent.to_builder().flatten().map;
            // intended behavior: replacing parents with children. see std doc
            map.extend(self.map);
            let max_index = map.iter().map(|(&i, _)| i).max().unwrap_or(usize::MIN);
            DescriptorSetBuilder {
                map,
                registry_marker: parent.registry_marker,
                max_index,
                parent: None,
                _marker: PhantomData,
            }
        } else {
            DescriptorSetBuilder {
                map: self.map,
                registry_marker: self.registry_marker,
                max_index: self.max_index,
                parent: None,
                _marker: PhantomData,
            }
        }
    }

    /// Builds this builder into a [`DescriptorSet`].
    pub fn build(self) -> DescriptorSet<'a, 'p> {
        DescriptorSet {
            registry_marker: self.registry_marker,
            inner: if self.map.is_empty() {
                DescriptorSetInner::Empty
            } else if self.max_index < SLICE_THRESHOLD {
                DescriptorSetInner::Slice(
                    (0usize..=self.max_index)
                        .map(|i| self.map.get(&i).copied().unwrap_or(std::ptr::null()))
                        .collect(),
                )
            } else {
                DescriptorSetInner::Map(self.map)
            },
            parent: self.parent,
            _marker: PhantomData,
        }
    }
}

impl Default for DescriptorSetBuilder<'_, '_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, 'p> From<DescriptorSetBuilder<'a, 'p>> for DescriptorSet<'a, 'p> {
    #[inline]
    fn from(value: DescriptorSetBuilder<'a, 'p>) -> Self {
        value.build()
    }
}

#[derive(Debug, Clone)]
enum DescriptorSetInner {
    Slice(Vec<*const ()>),
    Map(IHashMap<usize, *const ()>),
    Empty,
}

impl DescriptorSetInner {
    #[inline]
    fn get(&self, index: usize) -> *const () {
        match self {
            Self::Slice(v) => v.get(index).copied().unwrap_or(std::ptr::null()),
            Self::Map(m) => m.get(&index).copied().unwrap_or(std::ptr::null()),
            Self::Empty => std::ptr::null(),
        }
    }

    #[inline]
    fn contains(&self, index: usize) -> bool {
        match self {
            Self::Slice(v) => v.get(index).copied().is_some_and(|ptr| !ptr.is_null()),
            Self::Map(m) => m.contains_key(&index),
            Self::Empty => false,
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        match self {
            Self::Slice(v) => v.is_empty(),
            Self::Map(m) => m.is_empty(),
            Self::Empty => true,
        }
    }
}

/// A registry of descriptor types.
///
/// *Implementation note:*
///
/// It's strongly recommended to generate index for each descriptor from zero to N-1,
/// where N is the number of descriptor types registered.
///
/// # Safety
///
/// The implementation must guarantee that the returned index is unique for each type,
/// no registration overrides, and that the marker pointer is unique across registries.
pub unsafe trait Registry {
    /// Identifier of a descriptor type.
    type Identifier;

    /// Registers a descriptor type and identifier, returning its unique index if successful.
    fn register<T>(&mut self, id: Self::Identifier) -> Option<usize>;

    /// Returns a pointer to the unique marker of registry this registry refers to.
    fn marker(&self) -> *const ();
}

unsafe impl Send for DescriptorSet<'_, '_> {}
unsafe impl Sync for DescriptorSet<'_, '_> {}
unsafe impl Send for DescriptorSetBuilder<'_, '_> {}
unsafe impl Sync for DescriptorSetBuilder<'_, '_> {}
unsafe impl<T> Send for Type<T> {}
unsafe impl<T> Sync for Type<T> {}

/// Objects that support holding a descriptor set.
pub trait HoldDescriptors<'a, 'p> {
    /// Returns the descriptor set of this object.
    fn descriptors(&self) -> &DescriptorSet<'a, 'p>;
}

#[cfg(test)]
mod tests;
