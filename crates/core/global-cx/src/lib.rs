//! Rimecraft global context traits.
//!
//! # What is a global context?
//!
//! A global context (`Cx` in convention), is a global types and behavior provider for the whole
//! game. It provides types and behaviors that are used across the whole game, such as the
//! identifier type, the palette provider, and so on.

#![no_std]

extern crate alloc;

use alloc::boxed::Box;

/// Marker trait for global contexts.
pub trait GlobalContext: Sized + 'static {}

/// Marker trait for global contexts that provide an identifier type.
pub trait ProvideIdTy: GlobalContext {
    /// Identifier type.
    type Id;
}

/// Marker trait for global contexts that provide a `NbtCompound` type and friends.
pub trait ProvideNbtTy: GlobalContext {
    /// NBT compound type.
    type Compound;

    /// [`i32`] array type.
    type IntArray: Into<Box<[i32]>> + From<Box<[i32]>>;

    /// [`i64`] array type.
    type LongArray: Into<Box<[i64]>> + From<Box<[i32]>>;
}
