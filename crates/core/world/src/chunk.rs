//! Types and traits for working with chunks of blocks in a world.

mod internal_types;
mod section;
mod upgrade;

use std::fmt::Debug;

pub use internal_types::*;
use rimecraft_block::{ProvideBlockStateExtTy, ProvideStateIds, RawBlock};
use rimecraft_chunk_palette::{
    container::ProvidePalette, IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw, Maybe,
};
use rimecraft_fluid::ProvideFluidStateExtTy;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{ProvideRegistry, Registry};
pub use rimecraft_voxel_math::ChunkPos;
pub use section::ChunkSection;
pub use upgrade::UpgradeData;

use crate::view::HeightLimit;

/// Types associated with a `Chunk`.
///
/// # Generics
///
/// - `'w`: The world lifetime. See the crate document for more information.
pub trait ChunkCx<'w>: ProvideBlockStateExtTy + ProvideFluidStateExtTy + ProvideIdTy {
    /// The type of block state id list.
    type BlockStateList;

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
pub struct Chunk<'w, T, Cx>
where
    Cx: ChunkCx<'w>,
{
    pos: ChunkPos,
    udata: UpgradeData<'w, Cx>,
    hlimit: HeightLimit,
    section_array: Vec<ChunkSection<'w, Cx>>,

    vdata: T,
}

impl<'w, T, Cx> Chunk<'w, T, Cx>
where
    Cx: ChunkCx<'w>
        + ProvideStateIds<List = Cx::BlockStateList>
        + ProvidePalette<Cx::BlockStateList, IBlockState<'w, Cx>>
        + ProvidePalette<Cx::BiomeList, IBiome<'w, Cx>>
        + ProvideRegistry<'w, Cx::Id, RawBlock<'w, Cx>>,

    Cx::BlockStateList: for<'a> PalIndexToRaw<&'a IBlockState<'w, Cx>>
        + for<'s> PalIndexFromRaw<'s, Maybe<'s, IBlockState<'w, Cx>>>
        + Clone,

    &'w Registry<Cx::Id, Cx::Biome>: Into<Cx::BiomeList>,
    Cx::BiomeList: for<'a> PalIndexToRaw<&'a IBiome<'w, Cx>>
        + for<'s> PalIndexFromRaw<'s, Maybe<'s, IBiome<'w, Cx>>>
        + Clone,
{
    /// Creates a new chunk from scratch.
    ///
    /// # Panics
    ///
    /// Panics if the given section array length is not the vertical count of
    /// chunk sections of given `height_limit`.
    ///
    /// See [`HeightLimit::count_vertical_sections`].
    pub fn new<I>(
        pos: ChunkPos,
        upgrade_data: UpgradeData<'w, Cx>,
        height_limit: HeightLimit,
        biome_registry: &'w Registry<Cx::Id, Cx::Biome>,
        section_array: Option<I>,
        vdata: T,
    ) -> Self
    where
        I: Iterator<Item = Option<ChunkSection<'w, Cx>>> + ExactSizeIterator,
    {
        Self {
            pos,
            udata: upgrade_data,
            hlimit: height_limit,
            section_array: {
                let len = height_limit.count_vertical_sections() as usize;
                if let Some(section_array) = section_array {
                    assert_eq!(section_array.len(), len, "length of given section array should be the count of vertical sections of the chunk");

                    section_array
                        .into_iter()
                        .map(|opt| opt.unwrap_or_else(|| ChunkSection::from(biome_registry)))
                        .collect()
                } else {
                    (0..len)
                        .map(|_| ChunkSection::from(biome_registry))
                        .collect()
                }
            },
            vdata,
        }
    }
}

impl<'w, T, Cx> Debug for Chunk<'w, T, Cx>
where
    T: Debug,
    Cx: ChunkCx<'w> + Debug,
    Cx::Id: Debug,
    Cx::BlockStateExt: Debug,
    Cx::BlockStateList: Debug,
    Cx::FluidStateExt: Debug,
    Cx::Biome: Debug,
    Cx::BiomeList: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("pos", &self.pos)
            .field("udata", &self.udata)
            .field("hlimit", &self.hlimit)
            .field("section_array", &self.section_array)
            .field("vdata", &self.vdata)
            .finish()
    }
}
