#[cfg(feature = "arc")]
pub mod arc;

#[cfg(test)]
mod tests;

use std::{hash::Hash, ops::Deref};

use dashmap::DashSet;

/// Thread safe and hash-based caches.
///
/// A caches is a collection that provide cached value of
/// a given value to reduce memory usage.
pub struct Caches<T>
where
    T: Hash + Eq,
{
    map: DashSet<Box<T>>,
}

impl<T> Caches<T>
where
    T: Hash + Eq,
{
    /// Creates a new caches.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Obtain a reference from cached values in this caches,
    /// and the provided value will be dropped.
    /// If an equaled value doesn't exist in this caches, the value
    /// will be leaked into heap.
    pub fn get(&self, value: T) -> &T {
        if let Some(v) = self.map.get(&value) {
            unsafe { &*(v.deref().deref() as *const T) }
        } else {
            let boxed = Box::new(value);
            let ptr = boxed.deref() as *const T;
            self.map.insert(boxed);

            unsafe { &*ptr }
        }
    }

    /// Whether this caches contains the value.
    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        self.map.contains(value)
    }
}

impl<T> Default for Caches<T>
where
    T: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Self {
            map: DashSet::new(),
        }
    }
}
