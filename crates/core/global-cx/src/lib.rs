//! Rimecraft global context traits.
//!
//! # What is a global context?
//!
//! A global context (`Cx` in convention), is a global types and behavior provider for the whole
//! game. It provides types and behaviors that are used across the whole game, such as the
//! identifier type, the palette provider, and so on.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

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
    type IntArray;
    /// [`i64`] array type.
    type LongArray;
}

#[cfg(feature = "std")]
/// NBT `edcode`-ing related marker traits.
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
