//! Rimecraft global context traits.
//!
//! # What is a global context?
//!
//! A global context (`Cx` in convention), is a global types and behavior provider for the whole
//! game. It provides types and behaviors that are used across the whole game, such as the
//! identifier type, the palette provider, and so on.

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::{fmt::Display, hash::Hash};

#[cfg(feature = "std")]
pub mod nbt;

#[cfg(feature = "edcode")]
pub mod edcode;

#[cfg(feature = "rand")]
pub mod rand;

/// Marker trait for global contexts.
///
/// # Safety
///
/// The type should be zero-sized, and should contains no valid instances,
/// as it is used as a marker trait, and this guarantees that the type is
/// FFI-safe.
pub unsafe trait GlobalContext: Sized + 'static {}

/// Marker trait for global contexts that provide an identifier type.
pub trait ProvideIdTy: GlobalContext {
    /// Identifier type.
    type Id: Display + Hash + Eq;
}

/// Marker trait for global contexts that provide a version type.
pub trait ProvideVersionTy: GlobalContext {
    /// Version type.
    type Version: Display + Hash + Eq;
}

/// Marker trait for global contexts that provide a `NbtCompound` type and friends.
#[cfg(feature = "nbt")]
pub trait ProvideNbtTy: GlobalContext {
    /// NBT compound type.
    type Compound: Default;

    /// [`i32`] array type.
    type IntArray: Into<alloc::boxed::Box<[i32]>> + From<alloc::boxed::Box<[i32]>>;

    /// [`i64`] array type.
    type LongArray: Into<alloc::boxed::Box<[i64]>> + From<alloc::boxed::Box<[i64]>>;

    /// Function that converts a `Compound` to a `Deserializer`.
    fn compound_to_deserializer(compound: &Self::Compound) -> impl serde::Deserializer<'_>;
}

#[deprecated = "use `nbt` feature instead"]
pub use nbt as nbt_edcode;
