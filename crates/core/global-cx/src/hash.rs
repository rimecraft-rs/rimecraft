//! Hasher merker types and traits for global context constraints.

use core::{hash::BuildHasher, marker::PhantomData};

/// Marker trait for global contexts that provide a hasher implementation.
pub trait ProviderHasherTy<Marker> {
    /// The random state type.
    type BuildHasher: BuildHasher + Default;
}

/// Marker for hash tables with numeric keys with `BITS` bits.
#[derive(Debug)]
pub struct Numeric<const BITS: u32, Marker> {
    _marker: PhantomData<Marker>,
}

/// Cryptographically-secure hashers.
///
/// This is usually the slowest but is theoretically unpredictable,
/// which is free from attacks.
#[derive(Debug)]
pub struct Crypto;

/// Seeded but not cryptographically-secure hashers.
///
/// Free from most attacks, and is faster than crypto ones.
#[derive(Debug)]
pub struct Seeded;

/// Hashers with no protection at all. You may not want to use this.
#[derive(Debug)]
pub struct Unsecure;

/// Extension trait for hash tables.
pub trait HashTableExt {
    /// Creates an empty hash table, with zeroed capacity.
    fn new() -> Self;

    /// Creates an empty hash table, with at least the given capacity.
    fn with_capacity(capacity: usize) -> Self;
}

#[cfg(feature = "std")]
mod imp_std {
    use core::hash::BuildHasher;
    use std::collections::{HashMap, HashSet};

    use crate::HashTableExt;

    impl<K, V, R> HashTableExt for HashMap<K, V, R>
    where
        R: BuildHasher + Default,
    {
        #[inline]
        fn new() -> Self {
            Self::default()
        }

        #[inline]
        fn with_capacity(capacity: usize) -> Self {
            Self::with_capacity_and_hasher(capacity, Default::default())
        }
    }

    impl<K, R> HashTableExt for HashSet<K, R>
    where
        R: BuildHasher + Default,
    {
        #[inline]
        fn new() -> Self {
            Self::default()
        }

        #[inline]
        fn with_capacity(capacity: usize) -> Self {
            Self::with_capacity_and_hasher(capacity, Default::default())
        }
    }
}
