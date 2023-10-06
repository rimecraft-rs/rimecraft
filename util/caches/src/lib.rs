#[cfg(feature = "arc")]
pub mod arc;

#[cfg(test)]
mod tests;

use std::hash::Hash;

use dashmap::DashMap;

#[cfg(feature = "arc")]
pub use arc::Caches as ArcCaches;

/// Thread safe and hash-based caches.
///
/// A caches is a collection that provide cached value of
/// a given value to reduce memory usage.
pub struct Caches<T>
where
    T: Hash + Eq + 'static,
{
    map: DashMap<&'static T, *const T>,
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
            unsafe { &**v }
        } else {
            let ptr = Box::into_raw(Box::new(value));
            let refe = unsafe { &*ptr };
            self.map.insert(refe, ptr as *const T);
            refe
        }
    }

    /// Whether this caches contains the value.
    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }
}

impl<T> Default for Caches<T>
where
    T: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
}

impl<T> Drop for Caches<T>
where
    T: Hash + Eq,
{
    fn drop(&mut self) {
        let map = std::mem::take(&mut self.map);
        for (_, v) in map {
            unsafe { drop(Box::from_raw(v as *mut T)) };
        }
    }
}

unsafe impl<T> Send for Caches<T> where T: Hash + Eq + Send {}
unsafe impl<T> Sync for Caches<T> where T: Hash + Eq + Sync {}
