//! Minecraft world implementation.
//!
//! World generation is not implemented.
//!
//! # The World Lifetime
//!
//! The world lifetime is `'w`, in common. It is the lifetime of the world itself,
//! and `BlockState`s, `FluidState`s and the `Biome` registry should be bound to this lifetime.

pub mod behave;
pub mod border;
pub mod chunk;
pub mod event;
pub mod heightmap;
pub mod tick;
pub mod view;

mod _impl;

pub use _impl::*;

use entity::EntityCx;
use local_cx::GlobalProvideLocalCxTy;
use maybe::Maybe;
use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_chunk_palette::{
    IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw, IntoIteratorRef,
};
use rimecraft_fluid::ProvideFluidStateExtTy;
use rimecraft_global_cx::{Hold, ProvideIdTy, ProvideNbtTy};
use rimecraft_voxel_math::direction::Direction;

use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Weak},
};

pub use ahash::{AHashMap, AHashSet};

/// Types associated with a `Chunk`.
///
/// # Generics
///
/// - `'w`: The world lifetime. See the crate document for more information.
pub trait WorldCx<'w>
where
    Self: ProvideBlockStateExtTy
        + ProvideFluidStateExtTy
        + ProvideIdTy
        + ProvideNbtTy
        + ProvideBlockStateExtTy<BlockStateExt<'w>: Hold<NestedBlockStateExt<'w>>>
        + GlobalProvideLocalCxTy
        + EntityCx<'w>,
{
    /// The type of block state id list.
    type BlockStateList: for<'s> PalIndexFromRaw<'s, Maybe<'s, BlockState<'w, Self>>>
        + for<'a> PalIndexToRaw<&'a BlockState<'w, Self>>
        + for<'a> IntoIteratorRef<'a, Item = &'a BlockState<'w, Self>, IntoIter: ExactSizeIterator>
        + Clone;

    /// The type of biomes.
    type Biome: 'w;

    /// The type of biome id list.
    type BiomeList;

    /// The `Heightmap.Type` type of heightmaps.
    type HeightmapType: heightmap::Type<'w, Self> + Hash + Eq;

    /// The extension type of world chunks.
    type WorldChunkExt;
}

/// A marker type for invariant lifetime marking.
#[allow(missing_debug_implementations)] // should not have an instance
pub struct InvariantLifetime<'a>(PhantomData<fn(&'a ()) -> &'a ()>);

/// The default max light level of Minecraft.
pub const DEFAULT_MAX_LIGHT_LEVEL: u32 = 15;

/// A trait for types that can provide access to an [`Arc`] or [`Weak`].
///
/// This is most useful for self-referential types but need lazy access due to performance considerations.
pub trait ArcAccess<T: ?Sized> {
    /// Returns an [`Arc`] to the wrapped value.
    fn access_arc(self) -> Arc<T>;

    /// Returns a [`Weak`] to the wrapped value.
    fn access_weak(self) -> Weak<T>;
}

impl<T: ?Sized> ArcAccess<T> for Arc<T> {
    #[inline]
    fn access_arc(self) -> Arc<T> {
        self
    }

    #[inline]
    fn access_weak(self) -> Weak<T> {
        Arc::downgrade(&self)
    }
}

impl<T: ?Sized> ArcAccess<T> for &Arc<T> {
    #[inline]
    fn access_arc(self) -> Arc<T> {
        self.clone()
    }

    #[inline]
    fn access_weak(self) -> Weak<T> {
        Arc::downgrade(self)
    }
}

impl<T: ?Sized> ArcAccess<T> for Weak<T> {
    #[inline]
    fn access_arc(self) -> Arc<T> {
        self.upgrade().expect("wrapped value was dropped")
    }

    #[inline]
    fn access_weak(self) -> Weak<T> {
        self
    }
}

impl<T: ?Sized> ArcAccess<T> for &Weak<T> {
    #[inline]
    fn access_arc(self) -> Arc<T> {
        self.upgrade().expect("wrapped value was dropped")
    }

    #[inline]
    fn access_weak(self) -> Weak<T> {
        self.clone()
    }
}

/// An utility trait for reborrowing value.
pub trait Reclaim {
    /// The reborrowed value, which can be reborrowed again infinitely.
    type Output<'a>: Reclaim<Output<'a> = Self::Output<'a>>
    where
        Self: 'a;

    /// Reborrows the value.
    fn reclaim(&mut self) -> Self::Output<'_>;
}

impl<'env, T: ?Sized> Reclaim for &'env T {
    type Output<'a>
        = &'env T
    where
        Self: 'a;

    #[inline]
    fn reclaim(&mut self) -> Self::Output<'_> {
        *self
    }
}

impl<T: ?Sized> Reclaim for &mut T {
    type Output<'a>
        = &'a mut T
    where
        Self: 'a;

    #[inline]
    fn reclaim(&mut self) -> Self::Output<'_> {
        *self
    }
}

/// A subset of block state extensions type for use of World OPs.
#[derive(Debug)]
#[non_exhaustive]
pub struct NestedBlockStateExt<'a> {
    culling_shape: Arc<voxel_shape::Slice<'a>>,
    /// Opacity of this block state.
    opacity: u8,
    culling_faces: [Arc<voxel_shape::Slice<'a>>; Direction::COUNT],
    luminance: u32,
}

impl<'a> NestedBlockStateExt<'a> {
    /// Returns the culling face of the given direction.
    #[inline]
    pub fn culling_face(&self, direction: Direction) -> &Arc<voxel_shape::Slice<'a>> {
        &self.culling_faces[direction.ordinal()]
    }

    /// Returns the culling shape of this block state.
    #[inline]
    pub fn culling_shape(&self) -> &Arc<voxel_shape::Slice<'a>> {
        &self.culling_shape
    }

    /// Returns the opacity of this block state.
    #[inline]
    pub fn opacity(&self) -> u8 {
        self.opacity
    }

    /// Returns the luminance of this block state.
    #[inline]
    pub fn luminance(&self) -> u32 {
        self.luminance
    }

    /// Initializes the cached shape and friends of this block state.
    pub fn init_cache(&self) {
        todo!()
    }
}

impl Default for NestedBlockStateExt<'_> {
    fn default() -> Self {
        Self {
            culling_shape: voxel_shape::full_cube().clone(),
            culling_faces: std::array::from_fn(|_| voxel_shape::full_cube().clone()),
            opacity: 15,
            luminance: 0,
        }
    }
}

/// The runtime environment of an instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum Environment {
    /// The server environment.
    Server,
    /// The client environment.
    Client,
}

impl Environment {
    /// Whether this environment is the server environment.
    #[inline]
    pub const fn is_server(self) -> bool {
        matches!(self, Self::Server)
    }

    /// Whether this environment is the client environment.
    #[inline]
    pub const fn is_client(self) -> bool {
        matches!(self, Self::Client)
    }
}
