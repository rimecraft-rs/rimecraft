//! Chunk views.

use std::ops::{Deref, DerefMut};

use local_cx::{HoldLocalContext, LocalContext, LocalContextExt as _};
use rimecraft_voxel_math::{ChunkPos, ChunkSectionPos};

use crate::{
    LightType, WorldCx,
    chunk::{self, Chunk, ChunkStatus},
};

/// A view that provides set of chunks with locked access.
#[doc(alias = "ChunkProvider")]
pub trait ProvideLockedChunk<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// The chunk type.
    type Chunk<'a>
    where
        Self: 'a;

    /// Returns the chunk at the given chunk position.
    fn chunk<'a>(&'a self, pos: ChunkPos) -> Option<impl Deref<Target = Self::Chunk<'a>>>
    where
        Self: 'a;
}

/// View of chunks with more access options compared to [`ProvideLockedChunk`].
///
/// This view is intended to be high-level as a singular existence per-world so the methods
/// all take immutable references and inner-mutability is intended.
///
/// Corresponds to the `ChunkManager` in vanilla Minecraft.
#[doc(alias = "ChunkManager")]
pub trait ChunkView<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// The chunk type.
    type Chunk;

    /// Returns the chunk at the given chunk position and the least-required status.
    fn chunk(
        &self,
        pos: ChunkPos,
        least: ChunkStatus<'w, Cx>,
    ) -> Option<impl Deref<Target = Self::Chunk>>;

    /// Returns the lock-free accessible chunk at the given chunk position and the least-required status.
    fn chunk_mut(
        &self,
        pos: ChunkPos,
        least: ChunkStatus<'w, Cx>,
    ) -> Option<impl DerefMut<Target = Self::Chunk>>;

    /// Returns the chunk at the given chunk position and the least-required status,
    /// or introduces it into the view if absent.
    #[doc(alias = "chunk_or_create")]
    fn chunk_or_load(
        &self,
        pos: ChunkPos,
        least: ChunkStatus<'w, Cx>,
    ) -> impl Deref<Target = Self::Chunk>;

    /// Returns the lock-free accessible chunk at the given chunk position and the least-required status,
    /// or introduces it into the view if absent.
    #[doc(alias = "chunk_or_create_mut")]
    fn chunk_or_load_mut(
        &self,
        pos: ChunkPos,
        least: ChunkStatus<'w, Cx>,
    ) -> impl DerefMut<Target = Self::Chunk>;

    /// Whether the chunk at the given chunk position is already loaded into this view.
    fn is_chunk_loaded(&self, x: i32, z: i32) -> bool;
}

/// [`ChunkView`] along with mutable (locked) operations.
pub trait ChunkViewMut<'w, Cx>: ChunkView<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// Ticks the view.
    ///
    /// The `tick_notifier` is for supplying boolean values indicating whether the view should keep ticking.
    fn tick<F>(&self, tick_notifier: F, tick_chunks: bool)
    where
        F: FnMut() -> bool;

    /// Called when a chunk section occurs a light update.
    #[inline]
    #[doc(alias = "on_light_update")]
    fn light_update(&self, ty: LightType, pos: ChunkSectionPos) {
        // do nothing by default
        let _ = ty;
        let _ = pos;
    }
}

/// The default implementation of [`ProvideLockedChunk`] on a [`ChunkView`].
/// The obtained chunks are all locked variant in this case.
impl<'w, T, Cx> ProvideLockedChunk<'w, Cx> for T
where
    T: ChunkView<'w, Cx> + HoldLocalContext,
    T::LocalCx: LocalContext<chunk::status::Full<'w, Cx>>,
    for<'e> &'e <T as ChunkView<'w, Cx>>::Chunk: Chunk<'w, Cx>,
    Cx: WorldCx<'w>,
{
    type Chunk<'a>
        = <T as ChunkView<'w, Cx>>::Chunk
    where
        for<'e> &'e Self::Chunk<'a>: Chunk<'w, Cx>,
        Self: 'a;

    #[inline]
    fn chunk<'a>(&'a self, pos: ChunkPos) -> Option<impl Deref<Target = Self::Chunk<'a>>>
    where
        Self: 'a,
    {
        let full_status = self
            .local_context()
            .acquire_within::<chunk::status::Full<'w, Cx>>()
            .0;
        self.chunk(pos, full_status)
    }
}
