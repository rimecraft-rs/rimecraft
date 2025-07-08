//! General-purpose descriptor set system for Rust.

use std::{fmt::Debug, marker::PhantomData};

use ident_hash::{HashTableExt as _, IHashMap};

#[cfg(any(feature = "simple-registry", test))]
mod simple_registry;

#[cfg(feature = "simple-registry")]
pub use simple_registry::SimpleRegistry;

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
#[derive(Debug)]
pub struct DescriptorSet<'a> {
    inner: DescriptorSetInner,
    registry_marker: *const (),
    // role of lifetime: constrain this type not to outlive it.
    _marker: PhantomData<&'a ()>,
}

/// Builder of a [`DescriptorSet`].
#[derive(Debug)]
pub struct DescriptorSetBuilder<'a> {
    map: IHashMap<usize, *const ()>,
    registry_marker: *const (),
    max_index: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> DescriptorSet<'a> {
    /// Returns builder of this type.
    #[inline]
    pub fn builder() -> DescriptorSetBuilder<'a> {
        Default::default()
    }

    /// Returns the value of the given type if present.
    ///
    /// # Panics
    ///
    /// Panics if the type is not under same registry as this set.
    pub fn get<T: Copy>(&self, ty: Type<T>) -> Option<T> {
        let _: () = Type::<T>::__TYPE_CHECK;
        assert_eq!(
            self.registry_marker, ty.registry_marker,
            "not under the same registry"
        );
        let raw = self.inner.get(ty.index);
        (!raw.is_null()).then(|| {
            let borrowed = &raw;
            let ptr = std::ptr::from_ref(borrowed) as *const T;
            unsafe { *ptr }
        })
    }
}

impl<'a> DescriptorSetBuilder<'a> {
    /// Creates a new descriptor set builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: IHashMap::new(),
            registry_marker: std::ptr::null(),
            max_index: usize::MIN,
            _marker: PhantomData,
        }
    }

    /// Inserts a value into the descriptor set, returns if the insertion was successful.
    ///
    /// # Panics
    ///
    /// Panics if the type is not under same registry as the builder.
    pub fn insert<T: Copy + 'a>(&mut self, ty: Type<T>, value: T) -> bool {
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

    /// Builds this builder into a [`DescriptorSet`].
    pub fn build(self) -> DescriptorSet<'a> {
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
            _marker: PhantomData,
        }
    }
}

impl Default for DescriptorSetBuilder<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<DescriptorSetBuilder<'a>> for DescriptorSet<'a> {
    #[inline]
    fn from(value: DescriptorSetBuilder<'a>) -> Self {
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
            DescriptorSetInner::Slice(v) => v.get(index).copied().unwrap_or(std::ptr::null()),
            DescriptorSetInner::Map(m) => m.get(&index).copied().unwrap_or(std::ptr::null()),
            DescriptorSetInner::Empty => std::ptr::null(),
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

unsafe impl Send for DescriptorSet<'_> {}
unsafe impl Sync for DescriptorSet<'_> {}
unsafe impl Send for DescriptorSetBuilder<'_> {}
unsafe impl Sync for DescriptorSetBuilder<'_> {}
unsafe impl<T> Send for Type<T> {}
unsafe impl<T> Sync for Type<T> {}

#[cfg(test)]
mod tests;
