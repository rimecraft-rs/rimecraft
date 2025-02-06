//! Minecraft world implementation.
//!
//! World generation is not implemented.
//!
//! # The World Lifetime
//!
//! The world lifetime is `'w`, in common. It is the lifetime of the world itself,
//! and `BlockState`s, `FluidState`s and the `Biome` registry should be bound to this lifetime.

pub mod chunk;
pub mod event;
pub mod heightmap;
pub mod tick;
pub mod view;

pub mod behave;

use std::sync::Arc;

pub use ahash::{AHashMap, AHashSet};
use parking_lot::Mutex;
use rimecraft_block_entity::BlockEntity;

/// The default max light level of Minecraft.
pub const DEFAULT_MAX_LIGHT_LEVEL: u32 = 15;

/// A sealed cell.
#[derive(Debug)]
#[repr(transparent)]
pub struct Sealed<T>(pub(crate) T);

impl<T> From<T> for Sealed<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Boxed block entity cell with internal mutability and reference-counting.
pub type BlockEntityCell<'w, Cx> = Arc<Mutex<Box<BlockEntity<'w, Cx>>>>;
