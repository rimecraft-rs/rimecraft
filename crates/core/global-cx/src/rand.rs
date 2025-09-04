//! `rand` crate integration.

use core::ops::DerefMut;

pub use rand::{Rng, RngCore};

use crate::GlobalContext;

/// Global contexts providing random number generators.
pub trait ProvideRng: GlobalContext {
    /// Type of fast RNG.
    type FastRng: RngCore + 'static;

    /// Type of cryptographic RNG.
    type CryptoRng: RngCore + 'static;

    /// Returns a locked fast RNG.
    fn fast_rng() -> impl LockedRng<Self::FastRng>;

    /// Returns a locked cryptographic RNG.
    fn crypto_rng() -> impl LockedRng<Self::CryptoRng>;
}

/// A type that represents a locked RNG.
pub trait LockedRng<T> {
    /// Guarded access to the RNG.
    type Guard<'a>: DerefMut<Target = T> + 'a
    where
        Self: 'a;

    /// Locks the RNG.
    fn lock(&self) -> Self::Guard<'_>;
}
