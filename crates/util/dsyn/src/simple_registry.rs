use std::{any::TypeId, hash::Hash};

use ahash::AHashSet;

/// A simple registry implementation.
#[derive(Debug, Clone)]
pub struct SimpleRegistry<K> {
    set: AHashSet<(K, TypeId)>,
    ptr: Box<usize>,
}

impl<K> SimpleRegistry<K> {
    /// Creates a new simple registry.
    #[inline]
    pub fn new() -> Self {
        Self {
            set: AHashSet::new(),
            ptr: Box::new(0),
        }
    }
}

impl<K> Default for SimpleRegistry<K> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<K> super::Registry for SimpleRegistry<K>
where
    K: Hash + Eq,
{
    type Identifier = K;

    fn register<T>(&mut self, id: Self::Identifier) -> Option<usize> {
        self.set
            .insert((id, typeid::of::<T>()))
            .then_some(self.set.len() - 1)
    }

    fn marker(&self) -> *const () {
        std::ptr::from_ref(self.ptr.as_ref()) as *const ()
    }
}
