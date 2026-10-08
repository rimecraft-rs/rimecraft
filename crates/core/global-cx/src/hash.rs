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
