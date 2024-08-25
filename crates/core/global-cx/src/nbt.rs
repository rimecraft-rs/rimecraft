//! NBT `edcode`-ing related marker traits.

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
