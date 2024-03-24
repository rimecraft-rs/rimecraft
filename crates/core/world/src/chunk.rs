//! Types and traits for working with chunks in a world.
//!
//! A chunk represents a scoped, mutable view of `Biome`s, [`BlockState`]s, [`FluidState`]s and [`BlockEntity`]s.

mod internal_types;
mod section;
mod upgrade;

use std::{fmt::Debug, hash::Hash};

use ahash::AHashMap;
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

pub use internal_types::*;

use crate::{
    heightmap::{self, Heightmap},
    view::{BlockLuminanceView, BlockView, HeightLimit},
};

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

    /// The `Heightmap.Type` type of heightmaps.
    type HeightmapType: heightmap::Type<'w, Self> + Hash + Eq;
}

/// A generic chunk data structure.
#[non_exhaustive]
pub struct BaseChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Position of this chunk.
    pub pos: ChunkPos,
    /// Upgrade data of this chunk.
    pub upgrade_data: UpgradeData<'w, Cx>,
    /// Heightmaps of this chunk.
    pub heightmaps: AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>,
    /// Height limit of this chunk.
    pub height_limit: HeightLimit,
    /// Map of block positions to block entities.
    pub block_entities: AHashMap<BlockPos, CompoundedBlockEntity<'w, Cx>>,
    /// The internal chunk sections.
    pub section_array: Box<[ChunkSection<'w, Cx>]>,

    inhabited_time: u64,
    /// Whether this chunk needs saving.
    pub needs_saving: bool,
}

impl<'w, Cx> Debug for BaseChunk<'w, Cx>
where
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
            .field("upgrade_data", &self.upgrade_data)
            .field("height_limit", &self.height_limit)
            .field("section_array", &self.section_array)
            .field("inhabited_time", &self.inhabited_time)
            .field("needs_saving", &self.needs_saving)
            .finish_non_exhaustive()
    }
}

impl<'w, Cx> BaseChunk<'w, Cx>
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
        inhabited_time: u64,
        section_array: Option<I>,
    ) -> Self
    where
        I: Iterator<Item = Option<ChunkSection<'w, Cx>>> + ExactSizeIterator,
    {
        Self {
            pos,
            needs_saving: false,
            inhabited_time,
            upgrade_data,
            height_limit,
            heightmaps: AHashMap::new(),
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
        }
    }
}

/// Boxed [`BlockEntity`] with NBT compound.
pub struct CompoundedBlockEntity<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The NBT compound.
    pub nbt: Cx::Compound,
    /// The block entity.
    pub be: Box<BlockEntity<'w, Cx>>,
}

impl<'w, Cx> Debug for CompoundedBlockEntity<'w, Cx>
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

/// Types that can represent an immutable [`BaseChunk`].
pub trait AsBaseChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Returns a [`BaseChunk`].
    fn as_base_chunk(&self) -> &BaseChunk<'w, Cx>;
}

/// Immutable chunk behaviors.
pub trait Chunk<'w, Cx>
where
    Self: AsBaseChunk<'w, Cx> + BlockView<'w, Cx> + BlockLuminanceView<'w, Cx>,
    Cx: ChunkCx<'w>,
{
    /// Returns the array of chunk sections of this chunk.
    #[inline]
    fn sections(&self) -> &[ChunkSection<'w, Cx>] {
        &self.as_base_chunk().section_array
    }

    /// Gets the [`ChunkSection`] at the given Y index of this chunk.
    #[inline]
    fn section(&self, index: usize) -> Option<&ChunkSection<'w, Cx>> {
        self.sections().get(index)
    }

    /// Returns the index of highest non-empty [`ChunkSection`] in this chunk.
    ///
    /// See [`ChunkSection::is_empty`].
    fn highest_non_empty_section(&self) -> Option<usize> {
        self.sections().iter().rposition(|s| !s.is_empty())
    }

    /// Returns the heightmaps of this chunk.
    #[inline]
    fn heightmaps(&self) -> &AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>> {
        &self.as_base_chunk().heightmaps
    }

    /// Returns the position of this chunk.
    #[inline]
    fn pos(&self) -> ChunkPos {
        self.as_base_chunk().pos
    }
}

impl<'w, Cx, T> AsBaseChunk<'w, Cx> for T
where
    T: AsRef<BaseChunk<'w, Cx>>,
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn as_base_chunk(&self) -> &BaseChunk<'w, Cx> {
        self.as_ref()
    }
}
