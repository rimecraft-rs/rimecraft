//! Chunk views.

use std::ops::{Deref, DerefMut};

use local_cx::{HoldLocalContext, LocalContext, LocalContextExt};
use rimecraft_block::BlockState;
use rimecraft_fluid::BsToFs;
use rimecraft_voxel_math::ChunkSectionPos;
use serde::Deserialize;

use crate::{
    LightType,
    chunk::{self, Chunk, ChunkCx, ChunkStatus, ComputeIndex, WorldChunk, WorldChunkLocalCx},
    event::ServerEventCallback,
};

/// A view that provides set of chunks.
#[doc(alias = "ChunkProvider")]
pub trait ProvideChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The chunk type.
    type Chunk<'a>
    where
        Self: 'a;

    /// Returns the chunk at the given **chunk position**.
    fn chunk<'a>(&'a self, x: i32, z: i32) -> Option<impl Deref<Target = Self::Chunk<'a>>>
    where
        for<'e> &'e Self::Chunk<'a>: Chunk<'w, Cx>,
        Self: 'a;
}

/// View of world chunks.
///
/// This view is intended to be high-level as a singular existence per-world so the methods
/// all take immutable references and inner-mutability is intended.
///
/// Corresponds to the `ChunkManager` in vanilla Minecraft.
#[doc(alias = "ChunkManager")]
pub trait WorldChunkView<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Returns the world chunk at the given **chunk position** and the least-required status.
    fn world_chunk(
        &self,
        x: i32,
        z: i32,
        least: ChunkStatus<'w, Cx>,
    ) -> Option<impl Deref<Target = WorldChunk<'w, Cx>>>;

    /// Returns the lock-free accessible world chunk at the given **chunk position** and the least-required status.
    fn world_chunk_mut(
        &self,
        x: i32,
        z: i32,
        least: ChunkStatus<'w, Cx>,
    ) -> Option<impl DerefMut<Target = WorldChunk<'w, Cx>>>;

    /// Returns the world chunk at the given **chunk position** and the least-required status,
    /// or introduces it into the view if absent.
    #[doc(alias = "world_chunk_or_create")]
    fn world_chunk_or_load(
        &self,
        x: i32,
        z: i32,
        least: ChunkStatus<'w, Cx>,
    ) -> impl Deref<Target = WorldChunk<'w, Cx>>;

    /// Returns the lock-free accessible world chunk at the given **chunk position** and the least-required status,
    /// or introduces it into the view if absent.
    #[doc(alias = "world_chunk_or_create_mut")]
    fn world_chunk_or_load_mut(
        &self,
        x: i32,
        z: i32,
        least: ChunkStatus<'w, Cx>,
    ) -> impl DerefMut<Target = WorldChunk<'w, Cx>>;

    /// Ticks the view.
    ///
    /// The `tick_notifier` is for supplying boolean values indicating whether the view should keep ticking.
    fn tick<F>(&self, tick_notifier: F, tick_chunks: bool)
    where
        F: FnMut() -> bool;

    /// Called when a chunk section occurs a light update.
    #[inline(always)]
    #[doc(alias = "on_light_update")]
    fn light_update(&self, ty: LightType, pos: ChunkSectionPos) {
        // do nothing by default
        let _ = ty;
        let _ = pos;
    }
}

impl<'w, T, Cx> ProvideChunk<'w, Cx> for T
where
    T: WorldChunkView<'w, Cx> + HoldLocalContext,
    T::LocalCx: LocalContext<chunk::status::Full<'w, Cx>>,
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerEventCallback<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
    type Chunk<'a>
        = WorldChunk<'w, Cx>
    where
        for<'e> &'e Self::Chunk<'a>: Chunk<'w, Cx>,
        Self: 'a;

    #[inline]
    fn chunk<'a>(&'a self, x: i32, z: i32) -> Option<impl Deref<Target = Self::Chunk<'a>>>
    where
        for<'e> &'e Self::Chunk<'a>: Chunk<'w, Cx>,
    {
        let full_status = self
            .local_context()
            .acquire_within::<chunk::status::Full<'w, Cx>>()
            .0;
        self.world_chunk(x, z, full_status)
    }
}
