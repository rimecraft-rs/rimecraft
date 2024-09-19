//! Nbt reading and writing trait with local context.

use std::io;

use global_cx::{
    nbt::{ReadNbt, UpdateNbt, WriteNbt},
    GlobalContext,
};

use crate::WithLocalCx;

/// Marker trait for global contexts that can write nbt tags to a [`io::Write`] object.
pub trait WriteNbtWithCx<T, Cx>: GlobalContext {
    /// Function that performs writing operation.
    ///
    /// # Errors
    ///
    /// I/O errors.
    fn write_nbt<W>(value: T, writer: WithLocalCx<W, Cx>) -> Result<(), io::Error>
    where
        W: io::Write;
}

/// Marker trait for global contexts that can read nbt tags from a [`io::Read`] object.
pub trait ReadNbtWithCx<T, Cx>: GlobalContext {
    /// Function that performs reading operation.
    ///
    /// # Errors
    ///
    /// I/O errors.
    fn read_nbt<R>(reader: WithLocalCx<R, Cx>) -> Result<T, io::Error>
    where
        R: io::Read;
}

/// Marker trait for global contexts that can update existing nbt tags from a [`io::Read`] object.
pub trait UpdateNbtWithCx<T: ?Sized, Cx>: GlobalContext {
    /// Function that performs updating operation.
    ///
    /// # Errors
    ///
    /// I/O errors.
    fn update_nbt<R>(value: &mut T, reader: WithLocalCx<R, Cx>) -> Result<(), io::Error>
    where
        R: io::Read;
}
