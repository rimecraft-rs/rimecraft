//! Types and traits for working with chunks in a world.
//!
//! A chunk represents a scoped, mutable view of `Biome`s, `BlockState`s, `FluidState`s and `BlockEntity`s.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

use ahash::AHashMap;
use local_cx::LocalContext;
use parking_lot::{Mutex, RwLock};
use rimecraft_block::{BlockState, RawBlock};
use rimecraft_block_entity::BlockEntityCell;
use rimecraft_chunk_palette::{
    IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw, Maybe, container::ProvidePalette,
};
use rimecraft_registry::Registry;
use rimecraft_voxel_math::{BlockPos, ChunkSectionPos};

use crate::{
    WorldCx,
    chunk::light::ChunkSkyLight,
    heightmap::Heightmap,
    view::{
        HeightLimit,
        block::{MutBlockEntityView, MutBlockView},
        light::{MutBlockLuminanceView, MutLightSourceView},
    },
};

mod internal_types;

mod be_tick;
pub mod iter;
pub mod light;
mod section;
pub mod status;
mod upgrade;

mod world_chunk;

pub use rimecraft_voxel_math::ChunkPos;

pub use internal_types::*;

pub use section::*;
pub use status::{ChunkStatus, ChunkType};
pub use upgrade::UpgradeData;
pub use world_chunk::*;

/// The length of the border of a chunk.
pub const BORDER_LEN: u32 = 16;

/// The height of a chunk section.
pub const SECTION_HEIGHT: u32 = BORDER_LEN;

/// A generic chunk data structure.
#[non_exhaustive]
pub struct BaseChunk<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// Position of this chunk.
    pub pos: ChunkPos,
    /// Upgrade data of this chunk.
    pub upgrade_data: UpgradeData<'w, Cx>,
    /// Heightmaps of this chunk.
    pub heightmaps: RwLock<AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>>,
    /// Height limit of this chunk.
    pub height_limit: HeightLimit,
    /// Map of block positions to block entities.
    pub block_entities: Mutex<AHashMap<BlockPos, BlockEntityCell<'w, Cx>>>,
    /// Map of block positions to block entities.
    pub block_entity_nbts: Mutex<AHashMap<BlockPos, Cx::Compound>>,
    /// The internal chunk sections.
    pub section_array: Box<[Mutex<ChunkSection<'w, Cx>>]>,
    /// Increases for each tick a player spends with the chunk loaded.
    /// This is a cumulative measure of time.
    pub inhabited_time: u64,
    /// Whether this chunk needs saving.
    pub needs_saving: AtomicBool,
    /// The propagated sky light levels.
    pub sky_light: Mutex<ChunkSkyLight>,
}

impl<'w, Cx> Debug for BaseChunk<'w, Cx>
where
    Cx: WorldCx<'w> + Debug,
    Cx::Id: Debug,
    Cx::BlockStateExt<'w>: Debug,
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
    Cx: WorldCx<'w>
        + ProvidePalette<Cx::BlockStateList, IBlockState<'w, Cx>>
        + ProvidePalette<Cx::BiomeList, IBiome<'w, Cx>>,
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
    pub fn new<I, Local>(
        pos: ChunkPos,
        upgrade_data: UpgradeData<'w, Cx>,
        height_limit: HeightLimit,
        inhabited_time: u64,
        section_array: Option<I>,
        cx: Local,
    ) -> Self
    where
        I: Iterator<Item = Option<ChunkSection<'w, Cx>>> + ExactSizeIterator,
        Local: LocalContext<&'w Registry<Cx::Id, Cx::Biome>>
            + LocalContext<&'w Registry<Cx::Id, RawBlock<'w, Cx>>>
            + LocalContext<Cx::BlockStateList>,
    {
        Self {
            pos,
            needs_saving: AtomicBool::new(false),
            inhabited_time,
            upgrade_data,
            height_limit,
            heightmaps: RwLock::new(AHashMap::new()),
            block_entities: Mutex::new(AHashMap::new()),
            block_entity_nbts: Mutex::new(AHashMap::new()),
            section_array: {
                let len = height_limit.count_vertical_sections();
                if let Some(section_array) = section_array {
                    assert_eq!(
                        section_array.len(),
                        len,
                        "length of given section array should be the count of vertical sections of the chunk"
                    );
                    section_array
                        .map(|opt| {
                            Mutex::new(opt.unwrap_or_else(|| ChunkSection::from_registries(cx)))
                        })
                        .collect()
                } else {
                    (0..len)
                        .map(|_| Mutex::new(ChunkSection::from_registries(cx)))
                        .collect()
                }
            },
            sky_light: Mutex::new(ChunkSkyLight::new(height_limit)),
        }
    }
}

/// Types that can represent an access to a [`BaseChunk`].
pub trait AsBaseChunkAccess<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// The accessor type.
    type Access<'a>: BaseChunkAccess<'w, Cx>
    where
        Self: 'a;

    /// Returns an accessor to this chunk.
    fn as_base_chunk_access(&mut self) -> Self::Access<'_>;

    /// Returns a reference to the chunk.
    fn as_base_chunk(&self) -> &BaseChunk<'w, Cx>;
}

type SectionReadShorthand<'a, 'w, Chunk, Cx> =
    <<Chunk as AsBaseChunkAccess<'w, Cx>>::Access<'a> as BaseChunkAccess<'w, Cx>>::ChunkSectionRead;

/// Chunk behaviors.
pub trait Chunk<'w, Cx>
where
    Self: AsBaseChunkAccess<'w, Cx>
        + MutBlockView<'w, Cx>
        + MutBlockEntityView<'w, Cx>
        + MutBlockLuminanceView<'w, Cx>
        + MutLightSourceView<'w, Cx>,
    Cx: WorldCx<'w>,
{
    /// Returns the array of chunk sections of this chunk.
    #[inline]
    fn sections(&self) -> &[Mutex<ChunkSection<'w, Cx>>] {
        &self.as_base_chunk().section_array
    }

    /// Gets the [`ChunkSection`] at the given Y index of this chunk.
    #[inline]
    fn section(&self, index: usize) -> Option<&Mutex<ChunkSection<'w, Cx>>> {
        self.sections().get(index)
    }

    /// Returns the [`HeightLimit`] of this chunk.
    #[inline]
    fn height_limit(&self) -> HeightLimit {
        self.as_base_chunk().height_limit
    }

    /// Returns the index of highest non-empty [`ChunkSection`] in this chunk.
    ///
    /// See [`ChunkSection::is_empty`].
    fn highest_non_empty_section(&mut self) -> Option<usize> {
        self.as_base_chunk_access()
            .iter_read_chunk_sections()
            .into_iter()
            .rposition(|(_, s)| !s.is_empty())
    }

    /// Peeks the heightmaps of this chunk.
    #[inline]
    fn peek_heightmaps<F, T>(&mut self, pk: F) -> T
    where
        F: for<'a> FnOnce(&'a AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>) -> T,
    {
        let rg = self.as_base_chunk_access().read_heightmaps();
        pk(&rg)
    }

    /// Peeks the heightmaps of this chunk.
    #[inline]
    fn peek_heightmaps_mut<F, T>(&mut self, pk: F) -> T
    where
        F: for<'a> FnOnce(&'a mut AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>) -> T,
    {
        let mut rg = self.as_base_chunk_access().write_heightmaps();
        pk(&mut rg)
    }

    /// Returns the position of this chunk.
    #[inline]
    fn pos(&self) -> ChunkPos {
        self.as_base_chunk().pos
    }

    /// Refreshes the surface sky light propagated levels of this chunk.
    #[inline]
    fn refresh_surface_y(&mut self)
    where
        Cx: ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
    {
        BaseChunk::__csl_refresh_surface_y(self.as_base_chunk_access());
    }

    /// Returns an iterator over all blocks in this chunk.
    ///
    /// # Optimization
    ///
    /// The returned iterator is optimized for filtering, following vanilla behavior to do chunk section
    /// palette pre-checks for filtering out those sections that don't contain the desired block at all.
    ///
    /// # Panics
    ///
    /// Panics if the chunk has zero sections, which is not an intended state.
    #[allow(clippy::type_complexity)] // cant do better. help
    #[inline]
    fn blocks(
        &mut self,
    ) -> iter::Blocks<
        'w,
        impl DoubleEndedIterator<Item = (ChunkSectionPos, SectionReadShorthand<'_, 'w, Self, Cx>)>,
        SectionReadShorthand<'_, 'w, Self, Cx>,
        Cx,
    > {
        iter::blocks(self.as_base_chunk_access())
    }
}

#[allow(missing_docs)]
pub trait BaseChunkAccess<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    fn bca_as_bc(&self) -> &BaseChunk<'w, Cx>;

    type Reclaim<'borrow>: BaseChunkAccess<'w, Cx>
    where
        Self: 'borrow;

    fn reclaim(&mut self) -> Self::Reclaim<'_>;

    type HeighmapsRead: Deref<Target = AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>>;
    type HeighmapsWrite: DerefMut<Target = AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>>;
    type BlockEntitiesRead: Deref<Target = AHashMap<BlockPos, BlockEntityCell<'w, Cx>>>;
    type BlockEntitiesWrite: DerefMut<Target = AHashMap<BlockPos, BlockEntityCell<'w, Cx>>>;
    type BlockEntityNbtsRead: Deref<Target = AHashMap<BlockPos, Cx::Compound>>;
    type BlockEntityNbtsWrite: DerefMut<Target = AHashMap<BlockPos, Cx::Compound>>;
    type ChunkSectionRead: Deref<Target = ChunkSection<'w, Cx>>;
    type ChunkSectionWrite: DerefMut<Target = ChunkSection<'w, Cx>>;
    type ChunkSkyLightRead: Deref<Target = ChunkSkyLight>;
    type ChunkSkyLightWrite: DerefMut<Target = ChunkSkyLight>;

    fn read_heightmaps(self) -> Self::HeighmapsRead;
    fn write_heightmaps(self) -> Self::HeighmapsWrite;
    fn read_block_entities(self) -> Self::BlockEntitiesRead;
    fn write_block_entities(self) -> Self::BlockEntitiesWrite;
    fn read_block_entity_nbts(self) -> Self::BlockEntityNbtsRead;
    fn write_block_entity_nbts(self) -> Self::BlockEntityNbtsWrite;
    fn read_chunk_section(self, index: usize) -> Option<Self::ChunkSectionRead>;
    fn write_chunk_section(self, index: usize) -> Option<Self::ChunkSectionWrite>;
    #[allow(clippy::implied_bounds_in_impls)]
    fn iter_read_chunk_sections(
        self,
    ) -> impl Iterator<Item = (ChunkSectionPos, Self::ChunkSectionRead)>
    + DoubleEndedIterator
    + ExactSizeIterator;
    fn read_chunk_sky_light(self) -> Self::ChunkSkyLightRead;
    fn write_chunk_sky_light(self) -> Self::ChunkSkyLightWrite;

    fn mark_needs_saving(self);
    fn needs_saving(self) -> bool;
}

impl<'a, 'w, Cx: WorldCx<'w>> BaseChunkAccess<'w, Cx> for &'a BaseChunk<'w, Cx> {
    type HeighmapsRead =
        parking_lot::RwLockReadGuard<'a, AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>>;
    type HeighmapsWrite =
        parking_lot::RwLockWriteGuard<'a, AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>>;
    type BlockEntitiesRead = Self::BlockEntitiesWrite;
    type BlockEntitiesWrite =
        parking_lot::MutexGuard<'a, AHashMap<BlockPos, BlockEntityCell<'w, Cx>>>;
    type BlockEntityNbtsRead = Self::BlockEntityNbtsWrite;
    type BlockEntityNbtsWrite = parking_lot::MutexGuard<'a, AHashMap<BlockPos, Cx::Compound>>;
    type ChunkSectionRead = Self::ChunkSectionWrite;
    type ChunkSectionWrite = parking_lot::MutexGuard<'a, ChunkSection<'w, Cx>>;
    type ChunkSkyLightRead = Self::ChunkSkyLightWrite;
    type ChunkSkyLightWrite = parking_lot::MutexGuard<'a, ChunkSkyLight>;

    #[inline]
    fn read_heightmaps(self) -> Self::HeighmapsRead {
        self.heightmaps.read()
    }

    #[inline]
    fn write_heightmaps(self) -> Self::HeighmapsWrite {
        self.heightmaps.write()
    }

    #[inline]
    fn read_block_entities(self) -> Self::BlockEntitiesRead {
        self.write_block_entities()
    }

    #[inline]
    fn write_block_entities(self) -> Self::BlockEntitiesWrite {
        self.block_entities.lock()
    }

    #[inline]
    fn read_block_entity_nbts(self) -> Self::BlockEntityNbtsRead {
        self.write_block_entity_nbts()
    }

    #[inline]
    fn write_block_entity_nbts(self) -> Self::BlockEntityNbtsWrite {
        self.block_entity_nbts.lock()
    }

    #[inline]
    fn read_chunk_section(self, index: usize) -> Option<Self::ChunkSectionRead> {
        self.write_chunk_section(index)
    }

    #[inline]
    fn write_chunk_section(self, index: usize) -> Option<Self::ChunkSectionWrite> {
        self.section_array.get(index).map(Mutex::lock)
    }

    #[inline]
    #[allow(clippy::implied_bounds_in_impls)]
    fn iter_read_chunk_sections(
        self,
    ) -> impl Iterator<Item = (ChunkSectionPos, Self::ChunkSectionRead)>
    + DoubleEndedIterator
    + ExactSizeIterator {
        (self.height_limit.bottom_section_coord()..self.height_limit.top_section_coord())
            .zip(self.section_array.iter())
            .map(|(i, section)| ((self.pos, i).into(), section.lock()))
    }

    #[inline]
    fn read_chunk_sky_light(self) -> Self::ChunkSkyLightRead {
        self.write_chunk_sky_light()
    }

    #[inline]
    fn write_chunk_sky_light(self) -> Self::ChunkSkyLightWrite {
        self.sky_light.lock()
    }

    #[inline]
    fn bca_as_bc(&self) -> &BaseChunk<'w, Cx> {
        self
    }

    type Reclaim<'e>
        = Self
    where
        Self: 'e;

    #[inline]
    fn reclaim(&mut self) -> Self::Reclaim<'_> {
        *self
    }

    #[inline]
    fn mark_needs_saving(self) {
        self.needs_saving
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    fn needs_saving(self) -> bool {
        self.needs_saving.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl<'a, 'w, Cx: WorldCx<'w>> BaseChunkAccess<'w, Cx> for &'a mut BaseChunk<'w, Cx> {
    type HeighmapsRead = Self::HeighmapsWrite;
    type HeighmapsWrite = &'a mut AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>;
    type BlockEntitiesRead = Self::BlockEntitiesWrite;
    type BlockEntitiesWrite = &'a mut AHashMap<BlockPos, BlockEntityCell<'w, Cx>>;
    type BlockEntityNbtsRead = Self::BlockEntityNbtsWrite;
    type BlockEntityNbtsWrite = &'a mut AHashMap<BlockPos, Cx::Compound>;
    type ChunkSectionRead = Self::ChunkSectionWrite;
    type ChunkSectionWrite = &'a mut ChunkSection<'w, Cx>;
    type ChunkSkyLightRead = Self::ChunkSkyLightWrite;
    type ChunkSkyLightWrite = &'a mut ChunkSkyLight;

    #[inline]
    fn read_heightmaps(self) -> Self::HeighmapsRead {
        self.write_heightmaps()
    }

    #[inline]
    fn write_heightmaps(self) -> Self::HeighmapsWrite {
        self.heightmaps.get_mut()
    }

    #[inline]
    fn read_block_entities(self) -> Self::BlockEntitiesRead {
        self.write_block_entities()
    }

    #[inline]
    fn write_block_entities(self) -> Self::BlockEntitiesWrite {
        self.block_entities.get_mut()
    }

    #[inline]
    fn read_block_entity_nbts(self) -> Self::BlockEntityNbtsRead {
        self.write_block_entity_nbts()
    }

    #[inline]
    fn write_block_entity_nbts(self) -> Self::BlockEntityNbtsWrite {
        self.block_entity_nbts.get_mut()
    }

    #[inline]
    fn read_chunk_section(self, index: usize) -> Option<Self::ChunkSectionRead> {
        self.write_chunk_section(index)
    }

    #[inline]
    fn write_chunk_section(self, index: usize) -> Option<Self::ChunkSectionWrite> {
        self.section_array.get_mut(index).map(Mutex::get_mut)
    }

    #[inline]
    fn read_chunk_sky_light(self) -> Self::ChunkSkyLightRead {
        self.write_chunk_sky_light()
    }

    #[inline]
    fn write_chunk_sky_light(self) -> Self::ChunkSkyLightWrite {
        self.sky_light.get_mut()
    }

    #[inline]
    #[allow(clippy::implied_bounds_in_impls)]
    fn iter_read_chunk_sections(
        self,
    ) -> impl Iterator<Item = (ChunkSectionPos, Self::ChunkSectionRead)>
    + DoubleEndedIterator
    + ExactSizeIterator {
        (self.height_limit.bottom_section_coord()..self.height_limit.top_section_coord())
            .zip(self.section_array.iter_mut())
            .map(|(i, section)| ((self.pos, i).into(), section.get_mut()))
    }

    #[inline]
    fn bca_as_bc(&self) -> &BaseChunk<'w, Cx> {
        self
    }

    type Reclaim<'borrow>
        = &'borrow mut BaseChunk<'w, Cx>
    where
        Self: 'borrow;

    #[inline]
    fn reclaim(&mut self) -> Self::Reclaim<'_> {
        &mut **self
    }

    #[inline]
    fn mark_needs_saving(self) {
        *self.needs_saving.get_mut() = true;
    }

    #[inline]
    fn needs_saving(self) -> bool {
        *self.needs_saving.get_mut()
    }
}
