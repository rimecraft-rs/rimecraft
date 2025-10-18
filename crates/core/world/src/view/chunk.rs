//! Chunk views.

use rimecraft_voxel_math::ChunkSectionPos;

use crate::{
    LightType,
    chunk::{Chunk, ChunkCx},
};

/// A view that provides set of chunks.
#[doc(alias = "ChunkProvider")]
pub trait ProvideChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The chunk type.
    type Chunk<'a>: Chunk<'w, Cx>
    where
        Self: 'a;

    /// Returns the chunk at the given **chunk position**.
    fn chunk(&self, x: i32, z: i32) -> Option<Self::Chunk<'_>>;

    /// Called when a chunk section occurs a light update.
    #[inline]
    #[doc(alias = "on_light_update")]
    fn light_update(&self, ty: LightType, pos: ChunkSectionPos) {
        // do nothing
        let _ = ty;
        let _ = pos;
    }
}

/// View of chunks.
///
/// Corresponds to the `ChunkManager` in vanilla Minecraft.
#[doc(alias = "ChunkManager")]
pub trait WorldChunkView<'w, Cx>: ProvideChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
}
