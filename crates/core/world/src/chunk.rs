//! Types and traits for working with chunks in a world.
//!
//! A chunk represents a scoped, mutable view of `Biome`s, `BlockState`s, `FluidState`s and `BlockEntity`s.

use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use ahash::AHashMap;
use local_cx::{GlobalProvideLocalCxTy, LocalContext};
use parking_lot::{Mutex, RwLock};
use rimecraft_block::{BlockState, ProvideBlockStateExtTy, RawBlock};
use rimecraft_block_entity::BlockEntityCell;
use rimecraft_chunk_palette::{
    IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw, Maybe, container::ProvidePalette,
};
use rimecraft_fluid::ProvideFluidStateExtTy;
use rimecraft_global_cx::{ProvideIdTy, ProvideNbtTy};
use rimecraft_registry::Registry;
use rimecraft_voxel_math::BlockPos;

use crate::{
    Sealed,
    event::game_event,
    heightmap::{self, Heightmap},
    view::{
        HeightLimit,
        block::{BlockEntityView, BlockLuminanceView, BlockView},
    },
};

mod internal_types;

mod be_tick;
pub mod light;
mod section;
mod upgrade;

pub mod world_chunk;

pub use rimecraft_voxel_math::ChunkPos;

pub use section::ChunkSection;
pub use upgrade::UpgradeData;
pub use world_chunk::WorldChunk;

pub use internal_types::*;

/// The length of the border of a chunk.
pub const BORDER_LEN: u32 = 16;

/// Types associated with a `Chunk`.
///
/// # Generics
///
/// - `'w`: The world lifetime. See the crate document for more information.
pub trait ChunkCx<'w>
where
    Self: ProvideBlockStateExtTy
        + ProvideFluidStateExtTy
        + ProvideIdTy
        + ProvideNbtTy
        + GlobalProvideLocalCxTy,
{
    /// The type of block state id list.
    type BlockStateList: for<'s> PalIndexFromRaw<'s, Maybe<'s, BlockState<'w, Self>>>
        + for<'a> PalIndexToRaw<&'a BlockState<'w, Self>>
        + Clone;

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
            needs_saving: false,
            inhabited_time,
            upgrade_data,
            height_limit,
            heightmaps: RwLock::new(AHashMap::new()),
            block_entities: Mutex::new(AHashMap::new()),
            block_entity_nbts: Mutex::new(AHashMap::new()),
            section_array: {
                let len = height_limit.count_vertical_sections() as usize;
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
        }
    }
}

/// Types that can represent an immutable [`BaseChunk`].
pub trait AsBaseChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Returns a [`BaseChunk`].
    fn as_base_chunk(&self) -> Sealed<&BaseChunk<'w, Cx>>;
}

/// Types that can represent a mutable [`BaseChunk`].
pub trait AsBaseChunkMut<'w, Cx>: AsBaseChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Returns a [`BaseChunk`].
    fn as_base_chunk_mut(&mut self) -> Sealed<&mut BaseChunk<'w, Cx>>;
}

impl<'w, Cx, T> AsBaseChunk<'w, Cx> for &T
where
    T: AsBaseChunk<'w, Cx>,
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn as_base_chunk(&self) -> Sealed<&BaseChunk<'w, Cx>> {
        (**self).as_base_chunk()
    }
}

impl<'w, Cx, T> AsBaseChunk<'w, Cx> for &mut T
where
    T: AsBaseChunk<'w, Cx>,
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn as_base_chunk(&self) -> Sealed<&BaseChunk<'w, Cx>> {
        (**self).as_base_chunk()
    }
}

/// Immutable chunk behaviors.
///
/// _Implementation note:_ The default implementation of this trait is based on locking behavior.
/// Override `highest_non_empty_section`, `peek_heightmaps`, `peek_heightmaps_mut` to avoid it.
///
/// You may also want to override [`Chunk::peek_game_event_dispatcher`] in any case.
pub trait Chunk<'w, Cx>
where
    Self: AsBaseChunk<'w, Cx>
        + BlockView<'w, Cx>
        + BlockEntityView<'w, Cx>
        + BlockLuminanceView<'w, Cx>,
    Cx: ChunkCx<'w>,
{
    /// Returns the array of chunk sections of this chunk.
    #[inline]
    fn sections(&self) -> &[Mutex<ChunkSection<'w, Cx>>] {
        &self.as_base_chunk().0.section_array
    }

    /// Gets the [`ChunkSection`] at the given Y index of this chunk.
    #[inline]
    fn section(&self, index: usize) -> Option<&Mutex<ChunkSection<'w, Cx>>> {
        self.sections().get(index)
    }

    /// Returns the [`HeightLimit`] of this chunk.
    #[inline]
    fn height_limit(&self) -> HeightLimit {
        self.as_base_chunk().0.height_limit
    }

    /// Returns the index of highest non-empty [`ChunkSection`] in this chunk.
    ///
    /// See [`ChunkSection::is_empty`].
    fn highest_non_empty_section(self) -> Option<usize> {
        self.sections().iter().rposition(|s| !s.lock().is_empty())
    }

    /// Peeks the heightmaps of this chunk.
    #[inline]
    fn peek_heightmaps<F, T>(self, pk: F) -> T
    where
        F: for<'a> FnOnce(&'a AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>) -> T,
    {
        let rg = self.as_base_chunk().0.heightmaps.read();
        pk(&rg)
    }

    /// Peeks the heightmaps of this chunk.
    #[inline]
    fn peek_heightmaps_mut<F, T>(self, pk: F) -> T
    where
        F: for<'a> FnOnce(&'a mut AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>) -> T,
    {
        let mut rg = self.as_base_chunk().0.heightmaps.write();
        pk(&mut rg)
    }

    /// Returns the position of this chunk.
    #[inline]
    fn pos(&self) -> ChunkPos {
        self.as_base_chunk().0.pos
    }

    /// Peeks the [`game_event::Dispatcher`] of given Y section coordinate.
    #[inline]
    fn peek_game_event_dispatcher<F, T>(self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        let _ = y_section_coord;
        drop(f);
        None
    }

    /// Gets the [`game_event::Dispatcher`] of given Y section coordinate.
    #[inline]
    fn game_event_dispatcher(
        self,
        y_section_coord: i32,
    ) -> Option<Arc<game_event::Dispatcher<'w, Cx>>> {
        self.peek_game_event_dispatcher(y_section_coord, Arc::clone)
    }
}

/// Mutable chunk behaviors.
pub trait ChunkMut<'w, Cx>: Chunk<'w, Cx> + AsBaseChunkMut<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Returns the array of chunk sections of this chunk.
    #[inline]
    fn sections_mut(&mut self) -> &mut [Mutex<ChunkSection<'w, Cx>>] {
        &mut self.as_base_chunk_mut().0.section_array
    }

    /// Gets the [`ChunkSection`] at the given Y index of this chunk.
    #[inline]
    fn section_mut(&mut self, index: usize) -> Option<&mut Mutex<ChunkSection<'w, Cx>>> {
        self.sections_mut().get_mut(index)
    }

    /// Peeks the heightmaps of this chunk.
    #[inline]
    #[deprecated]
    fn peek_heightmaps_mut<F, T>(&mut self, pk: F) -> T
    where
        F: for<'a> FnOnce(&'a mut AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>) -> T,
    {
        pk(self.as_base_chunk_mut().0.heightmaps.get_mut())
    }
}

#[allow(unused)]
trait BaseChunkAccess<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    fn bca_as_bc(&self) -> &BaseChunk<'w, Cx>;
    fn reclaim(&mut self) -> impl BaseChunkAccess<'w, Cx>;

    type HeighmapsRead: Deref<Target = AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>>;
    type HeighmapsWrite: DerefMut<Target = AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>>;
    type BlockEntitiesRead: Deref<Target = AHashMap<BlockPos, BlockEntityCell<'w, Cx>>>;
    type BlockEntitiesWrite: DerefMut<Target = AHashMap<BlockPos, BlockEntityCell<'w, Cx>>>;
    type BlockEntityNbtsRead: Deref<Target = AHashMap<BlockPos, Cx::Compound>>;
    type BlockEntityNbtsWrite: DerefMut<Target = AHashMap<BlockPos, Cx::Compound>>;
    type ChunkSectionRead: Deref<Target = ChunkSection<'w, Cx>>;
    type ChunkSectionWrite: DerefMut<Target = ChunkSection<'w, Cx>>;

    fn read_heightmaps(self) -> Self::HeighmapsRead;
    fn write_heightmaps(self) -> Self::HeighmapsWrite;
    fn read_block_entities(self) -> Self::BlockEntitiesRead;
    fn write_block_entities(self) -> Self::BlockEntitiesWrite;
    fn read_block_entity_nbts(self) -> Self::BlockEntityNbtsRead;
    fn write_block_entity_nbts(self) -> Self::BlockEntityNbtsWrite;
    fn read_chunk_section(self, index: usize) -> Option<Self::ChunkSectionRead>;
    fn write_chunk_section(self, index: usize) -> Option<Self::ChunkSectionWrite>;
}

impl<'a, 'w, Cx: ChunkCx<'w>> BaseChunkAccess<'w, Cx> for &'a BaseChunk<'w, Cx> {
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
    fn bca_as_bc(&self) -> &BaseChunk<'w, Cx> {
        self
    }

    #[inline]
    fn reclaim(&mut self) -> impl BaseChunkAccess<'w, Cx> {
        *self
    }
}

impl<'a, 'w, Cx: ChunkCx<'w>> BaseChunkAccess<'w, Cx> for &'a mut BaseChunk<'w, Cx> {
    type HeighmapsRead = Self::HeighmapsWrite;
    type HeighmapsWrite = &'a mut AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>;
    type BlockEntitiesRead = Self::BlockEntitiesWrite;
    type BlockEntitiesWrite = &'a mut AHashMap<BlockPos, BlockEntityCell<'w, Cx>>;
    type BlockEntityNbtsRead = Self::BlockEntityNbtsWrite;
    type BlockEntityNbtsWrite = &'a mut AHashMap<BlockPos, Cx::Compound>;
    type ChunkSectionRead = Self::ChunkSectionWrite;
    type ChunkSectionWrite = &'a mut ChunkSection<'w, Cx>;

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
    fn bca_as_bc(&self) -> &BaseChunk<'w, Cx> {
        self
    }

    #[inline]
    fn reclaim(&mut self) -> impl BaseChunkAccess<'w, Cx> {
        &mut **self
    }
}
