//! Types and traits for working with chunks of blocks in a world.

mod internal_types;
mod section;
mod upgrade;

use std::fmt::Debug;

use ahash::AHashMap;
pub use internal_types::*;
use rimecraft_block::{ProvideBlockStateExtTy, ProvideStateIds, RawBlock};
use rimecraft_block_entity::BlockEntity;
use rimecraft_chunk_palette::{
    container::ProvidePalette, IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw, Maybe,
};
use rimecraft_fluid::ProvideFluidStateExtTy;
use rimecraft_global_cx::{ProvideIdTy, ProvideNbtTy};
use rimecraft_registry::{ProvideRegistry, Registry};
use rimecraft_voxel_math::BlockPos;
pub use rimecraft_voxel_math::ChunkPos;
pub use section::ChunkSection;
pub use upgrade::UpgradeData;

use crate::view::HeightLimit;

/// Types associated with a `Chunk`.
///
/// # Generics
///
/// - `'w`: The world lifetime. See the crate document for more information.
pub trait ChunkCx<'w>
where
    Self: ProvideBlockStateExtTy + ProvideFluidStateExtTy + ProvideIdTy + ProvideNbtTy,
{
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
/// - `Cx`: The global context type, providing access to the static fields and logics of the game.
pub struct Chunk<'w, T, Cx>
where
    Cx: ChunkCx<'w>,
{
    pos: ChunkPos,
    udata: UpgradeData<'w, Cx>,
    hlimit: HeightLimit,
    block_entities: AHashMap<BlockPos, BEWithCompound<'w, Cx>>,
    section_array: Box<[ChunkSection<'w, Cx>]>,

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
            block_entities: AHashMap::new(),
            section_array: {
                let len = height_limit.count_vertical_sections() as usize;
                if let Some(section_array) = section_array {
                    assert_eq!(section_array.len(), len, "length of given section array should be the count of vertical sections of the chunk");

                    section_array
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

impl<'w, T, Cx> Chunk<'w, T, Cx>
where
    Cx: ChunkCx<'w>,
    T: DataBehave<'w, Cx>,
{
    /// Returns the chunk section array of this chunk.
    ///
    /// This will be effected by the variant data. This should only happen with MCJE's `WrapperProtoChunk`.
    #[inline]
    pub fn sections(&self) -> &[ChunkSection<'w, Cx>] {
        self.vdata.sections(&self.section_array)
    }

    /// Searches for the highest non-empty `ChunkSection` of this chunk and returns its index.
    ///
    /// See [`ChunkSection::is_empty`].
    pub fn highest_non_empty_section(&self) -> Option<usize> {
        self.sections().iter().rposition(|sec| !sec.is_empty())
    }
}

impl<'w, T, Cx> Chunk<'w, T, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Gets the position of this chunk.
    #[inline]
    pub fn pos(&self) -> ChunkPos {
        self.pos
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

/// Common behaviors of a chunk variant data.
pub trait DataBehave<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Gets the chunk section slice, with the given internal section slice of [`Chunk`].
    #[inline(always)]
    fn sections<'a>(&'a self, super_secs: &'a [ChunkSection<'w, Cx>]) -> &'a [ChunkSection<'w, Cx>]
    where
        'w: 'a,
    {
        super_secs
    }
}

struct BEWithCompound<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    pub nbt: Cx::Compound,
    pub be: Box<BlockEntity<'w, Cx>>,
}

impl<'w, Cx> Debug for BEWithCompound<'w, Cx>
where
    Cx: ChunkCx<'w> + Debug,
    Cx::Compound: Debug,
    Cx::BlockStateExt: Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockEntityWithCompound")
            .field("nbt", &self.nbt)
            .field("be", &&self.be)
            .finish()
    }
}
