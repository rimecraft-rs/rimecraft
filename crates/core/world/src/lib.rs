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

use std::{
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

pub use ahash::{AHashMap, AHashSet};
use local_cx::dsyn::DescriptorTypeCache;
use parking_lot::Mutex;
use rimecraft_block_entity::{BlockEntity, BlockEntityConstructor};

use crate::chunk::ChunkCx;

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

//TODO: PLACEHOLDERS

/// Placeholder of type `ServerWorld`.
pub(crate) type ServerWorld<'w, Cx> = placeholder::ServerWorld<'w, Cx>;

/// Placeholder of type `World`.
pub(crate) type World<'w, Cx> = placeholder::World<'w, Cx>;

/// Placeholder of type `Entity`.
pub(crate) type Entity<'w, Cx> = placeholder::Entity<'w, Cx>; // Should be atomic reference counted with internal mutability.

#[allow(missing_docs, missing_debug_implementations)]
mod placeholder {
    use std::marker::PhantomData;

    pub struct ServerWorld<'w, Cx>(PhantomData<&'w Cx>);

    pub struct World<'w, Cx>(PhantomData<&'w Cx>);

    pub struct Entity<'w, Cx>(PhantomData<&'w Cx>);

    impl<Cx> Clone for Entity<'_, Cx> {
        fn clone(&self) -> Self {
            Self(PhantomData)
        }
    }
}

#[derive(Debug)]
pub(crate) struct DsynCache<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    be_constructor: OnceLock<dsyn::Type<BlockEntityConstructor<Cx>>>,
    _marker: PhantomData<&'w ()>,
}

impl<'w, Cx> DescriptorTypeCache<BlockEntityConstructor<Cx>> for DsynCache<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn get_or_cache<F>(&self, f: F) -> dsyn::Type<BlockEntityConstructor<Cx>>
    where
        F: FnOnce() -> dsyn::Type<BlockEntityConstructor<Cx>>,
    {
        *self.be_constructor.get_or_init(f)
    }
}
