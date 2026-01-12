//! Hasher that doesn't hash at all.

use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hasher},
};

/// A hasher that doesn't hash anything, but returns the given value,
/// which is useful for integral-keyed hash tables.
///
/// The hasher could only be used for integral types except `i128` and `u128`,
/// since [`Hasher`] doesn't support 128-bit output.
#[derive(Debug)]
#[repr(transparent)]
pub struct IdentityHasher {
    inner: u64,
}

impl IdentityHasher {
    /// Creates a new hasher.
    #[inline]
    pub fn new() -> Self {
        Self { inner: u64::MAX }
    }
}

impl Default for IdentityHasher {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_hash_for_identity_hasher {
    ($($f:ident,$t:ty),*$(,)?) => {
        $(
            #[inline]
            #[allow(trivial_numeric_casts)]
            fn $f(&mut self, value: $t) {
                self.inner = value as u64;
            }
        )*
    };
}

impl Hasher for IdentityHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.inner
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!("IdentityHasher can't hash bytes")
    }

    impl_hash_for_identity_hasher! {
        write_u8, u8,
        write_u16, u16,
        write_u32, u32,
        write_u64, u64,
        write_usize, usize,
        write_i8, i8,
        write_i16, i16,
        write_i32, i32,
        write_i64, i64,
    }
}

/// A build hasher that uses [`IdentityHasher`].
pub type IdentityBuildHasher = BuildHasherDefault<IdentityHasher>;

/// A hash map that uses [`IdentityHasher`].
pub type IHashMap<K, V> = HashMap<K, V, IdentityBuildHasher>;

/// A hash set that uses [`IdentityHasher`].
pub type IHashSet<K> = HashSet<K, IdentityBuildHasher>;

/// Extension trait for hash tables.
pub trait HashTableExt {
    /// Creates an empty hash table, with zeroed capacity.
    fn new() -> Self;

    /// Creates an empty hash table, with at least the given capacity.
    fn with_capacity(capacity: usize) -> Self;
}

impl<K, V> HashTableExt for IHashMap<K, V> {
    #[inline]
    fn new() -> Self {
        Self::with_hasher(IdentityBuildHasher::new())
    }

    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, IdentityBuildHasher::new())
    }
}

impl<K> HashTableExt for IHashSet<K> {
    #[inline]
    fn new() -> Self {
        Self::with_hasher(IdentityBuildHasher::new())
    }

    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, IdentityBuildHasher::new())
    }
}

#[cfg(test)]
mod tests;
