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
    type Compound;

    /// [`i32`] array type.
    type IntArray: Into<alloc::boxed::Box<[i32]>> + From<alloc::boxed::Box<[i32]>>;

    /// [`i64`] array type.
    type LongArray: Into<alloc::boxed::Box<[i64]>> + From<alloc::boxed::Box<[i32]>>;

    /// Function that converts a `Compound` to a `Deserializer`.
    fn compound_to_deserializer(compound: &Self::Compound) -> impl serde::Deserializer<'_>;
}

/// NBT `edcode`-ing related marker traits.
#[cfg(feature = "std")]
pub mod nbt_edcode {
    use std::io;

    use crate::GlobalContext;

    /// Marker trait for global contexts that can write nbt tags to a [`io::Write`] object.
    pub trait WriteNbt<T>: GlobalContext {
        /// Function that performs writing operation.
        ///
        /// # Errors
        ///
        /// I/O errors.
        fn write_nbt<W>(value: T, writer: W) -> Result<(), io::Error>
        where
            W: io::Write;
    }

    /// Marker trait for global contexts that can read nbt tags from a [`io::Read`] object.
    pub trait ReadNbt<T>: GlobalContext {
        /// Function that performs reading operation.
        ///
        /// # Errors
        ///
        /// I/O errors.
        fn read_nbt<R>(reader: R) -> Result<T, io::Error>
        where
            R: io::Read;
    }

    /// Marker trait for global contexts that can update existing nbt tags from a [`io::Read`] object.
    pub trait UpdateNbt<T: ?Sized>: GlobalContext {
        /// Function that performs updating operation.
        ///
        /// # Errors
        ///
        /// I/O errors.
        fn update_nbt<R>(value: &mut T, reader: R) -> Result<(), io::Error>
        where
            R: io::Read;
    }

    impl<T, Cx> UpdateNbt<T> for Cx
    where
        Cx: ReadNbt<T>,
    {
        #[inline]
        fn update_nbt<R>(value: &mut T, reader: R) -> Result<(), io::Error>
        where
            R: io::Read,
        {
            *value = Self::read_nbt(reader)?;
            Ok(())
        }
    }
}
