//! Chunk loading stage and built-in values abstraction.

use std::{fmt::Debug, sync::Arc};

use ahash::AHashSet;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

use crate::chunk::ChunkCx;

/// Type of a chunk specifying whether it is loaded into a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum ChunkType {
    /// A chunk which is incomplete and not loaded into a world yet.
    Proto,
    /// A chunk that is complete and bound to a world.
    Level,
}

/// The underlying type for registration [`ChunkStatus`].
pub struct RawChunkStatus<'w, Cx>(Arc<ChunkStatusInner<'w, Cx>>)
where
    Cx: ChunkCx<'w>;

/// Descriptor to loading status of a chunk.
///
/// Statuses of a chunk are ordered by their index to represent the loading process.
pub type ChunkStatus<'w, Cx> = Reg<'w, <Cx as ProvideIdTy>::Id, RawChunkStatus<'w, Cx>>;

struct ChunkStatusInner<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    index: usize,
    heightmap_types: AHashSet<Cx::HeightmapType>,
    ty: ChunkType,

    prev: Option<RawChunkStatus<'w, Cx>>,
}

impl<'w, Cx> RawChunkStatus<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Creates a new chunk status, increasing index value from the previous status or 0 if absent.
    pub fn new(
        previous: Option<&Self>,
        heightmap_types: AHashSet<Cx::HeightmapType>,
        ty: ChunkType,
    ) -> Self {
        Self(Arc::new(ChunkStatusInner {
            index: previous.map(|p| p.0.index + 1).unwrap_or(0),
            heightmap_types,
            ty,
            prev: previous.map(|p| Self(p.0.clone())),
        }))
    }

    /// Returns the index of this status.
    #[inline]
    pub fn index(&self) -> usize {
        self.0.index
    }

    /// Returns the previous status, if any or this value.
    ///
    /// The returned value is not wrapped within registration.
    #[inline]
    pub fn prev(&self) -> &Self {
        self.0.prev.as_ref().unwrap_or(self)
    }

    /// Returns the chunk type.
    #[inline]
    pub fn ty(&self) -> ChunkType {
        self.0.ty
    }

    /// Returns the heightmap types.
    #[inline]
    pub fn heightmap_types(&self) -> &AHashSet<Cx::HeightmapType> {
        &self.0.heightmap_types
    }
}

impl<'w, Cx> Debug for RawChunkStatus<'w, Cx>
where
    Cx: ChunkCx<'w>,
    Cx::HeightmapType: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkStatus")
            .field("index", &self.0.index)
            .field("heightmap_types", &self.0.heightmap_types)
            .field("type", &self.0.ty)
            .finish_non_exhaustive()
    }
}

// built-in local context extractors

/// A chunk status that is fully loaded.
pub struct Full<'w, Cx: ChunkCx<'w>>(pub ChunkStatus<'w, Cx>);

impl<'w, Cx: ChunkCx<'w>> Debug for Full<'w, Cx>
where
    Cx: ChunkCx<'w>,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Full").field(&self.0).finish()
    }
}
