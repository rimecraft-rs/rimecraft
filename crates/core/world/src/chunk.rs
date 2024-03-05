//! Types and traits for working with chunks of blocks in a world.

mod internal_types;
mod section;
mod upgrade;

use std::fmt::Debug;

pub use internal_types::*;
pub use rimecraft_voxel_math::ChunkPos;
pub use section::ChunkSection;
pub use upgrade::UpgradeData;

use crate::view::HeightLimit;

/// Types associated with a `Chunk`.
///
/// # Generics
///
/// - `'w`: The world lifetime. See the crate document for more information.
pub trait ChunkTy<'w> {
    /// The type of block state extensions.
    type BlockStateExt: 'w;
    /// The type of block state id list.
    type BlockStateList;

    /// The type of fluid state extensions.
    type FluidStateExt: 'w;

    /// The type of biomes.
    type Biome: 'w;
    /// The type of biome id list.
    type BiomeList;
}

/// A scoped, mutable view of biomes, block states, fluid states and
/// block entities.
///
/// # Generics
///
/// - `'w`: The world lifetime. See the crate document for more information.
/// - `T`: The chunk implementation data type. It provides functionalities like `WorldChunk` and `ProtoChunk`.
/// - `K`: The `Identifier` type.
/// - `Cx`: The global context type, providing access to the static fields and logics of the game.
pub struct Chunk<'w, T, K, Cx>
where
    Cx: ChunkTy<'w>,
{
    pos: ChunkPos,

    hlimit: HeightLimit,
    section_array: Option<Vec<ChunkSection<'w, K, Cx>>>,

    vdata: T,
}

impl<'w, T, K, Cx> Chunk<'w, T, K, Cx>
where
    Cx: ChunkTy<'w>,
{
    /// Creates a new chunk from scratch.
    pub fn new(
        pos: ChunkPos,
        height_limit: HeightLimit,
        section_array: Option<Vec<ChunkSection<'w, K, Cx>>>,
        vdata: T,
    ) -> Self {
        Self {
            pos,
            hlimit: height_limit,
            section_array,
            vdata,
        }
    }
}

impl<'w, T, K, Cx> Debug for Chunk<'w, T, K, Cx>
where
    T: Debug,
    K: Debug,
    Cx: ChunkTy<'w> + Debug,
    Cx::BlockStateExt: Debug,
    Cx::BlockStateList: Debug,
    Cx::Biome: Debug,
    Cx::BiomeList: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("pos", &self.pos)
            .field("hlimit", &self.hlimit)
            .field("section_array", &self.section_array)
            .field("vdata", &self.vdata)
            .finish()
    }
}