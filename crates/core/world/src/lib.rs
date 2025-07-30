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
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub use ahash::{AHashMap, AHashSet};
use parking_lot::Mutex;
use rimecraft_block_entity::BlockEntity;

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

    use crate::chunk::ChunkCx;

    pub struct ServerWorld<'w, Cx>(PhantomData<&'w Cx>);

    impl<'w, Cx> AsRef<World<'w, Cx>> for ServerWorld<'w, Cx>
    where
        Cx: ChunkCx<'w>,
    {
        fn as_ref(&self) -> &World<'w, Cx> {
            &World(PhantomData)
        }
    }

    pub struct World<'w, Cx>(PhantomData<&'w Cx>);

    pub struct Entity<'w, Cx>(PhantomData<&'w Cx>);

    impl<Cx> Clone for Entity<'_, Cx> {
        fn clone(&self) -> Self {
            Self(PhantomData)
        }
    }
}

/// A data joined with a world reference.
///
/// This is mainly used to deal with self-referencing in vanilla Minecraft, and is intended only
/// for temporary use as an intermediate trait implementor.
pub struct WorldJoined<'borrow, 'w, Cx, T>
where
    Cx: ChunkCx<'w>,
{
    pub(crate) world: &'borrow World<'w, Cx>,
    pub(crate) inner: T,
}

impl<'w, Cx, T> Debug for WorldJoined<'_, 'w, Cx, T>
where
    World<'w, Cx>: Debug,
    Cx: ChunkCx<'w>,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldJoined")
            .field("world", &self.world)
            .field("data", &self.inner)
            .finish()
    }
}

impl<'w, Cx, T> Clone for WorldJoined<'_, 'w, Cx, T>
where
    Cx: ChunkCx<'w>,
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            world: self.world,
            inner: self.inner.clone(),
        }
    }
}

impl<'w, Cx, T> Copy for WorldJoined<'_, 'w, Cx, T>
where
    Cx: ChunkCx<'w>,
    T: Copy,
{
}

impl<'w, Cx, T> Deref for WorldJoined<'_, 'w, Cx, T>
where
    Cx: ChunkCx<'w>,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'w, Cx, T> DerefMut for WorldJoined<'_, 'w, Cx, T>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub(crate) use __dsyn_cache::DsynCache;

mod __dsyn_cache {
    use crate::behave::*;

    use crate::chunk::ChunkCx;

    macro_rules! dsyn_caches_init {
    ($($f:ident=>$t:ident),*$(,)?) => {
        #[derive(Debug)]
        pub(crate) struct DsynCache<'w, Cx>
        where
            Cx: ChunkCx<'w>,
        {
            $($f: std::sync::OnceLock<dsyn::Type<$t<Cx>>>,)*
            _marker: std::marker::PhantomData<&'w ()>,
        }

        $(
        impl<'w, Cx> local_cx::dsyn::DescriptorTypeCache<$t<Cx>> for DsynCache<'w, Cx>
        where
            Cx: ChunkCx<'w>,
        {
            #[inline]
            fn get_or_cache<F>(&self, f: F) -> dsyn::Type<$t<Cx>>
            where
                F: FnOnce() -> dsyn::Type<$t<Cx>>,
            {
                *self.$f.get_or_init(f)
            }
        }
        )*
    };
    }

    dsyn_caches_init! {
        be_constructor => BlockEntityConstructor,
        be_on_block_replaced => BlockEntityOnBlockReplaced,
    }
}
