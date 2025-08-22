//! Types and traits for working with chunks in a world.
//!
//! A chunk represents a scoped, mutable view of `Biome`s, `BlockState`s, `FluidState`s and `BlockEntity`s.

use std::{fmt::Debug, hash::Hash, sync::Arc};

use ahash::AHashMap;
use local_cx::{GlobalProvideLocalCxTy, LocalContext};
use parking_lot::{Mutex, RwLock};
use rimecraft_block::{BlockState, ProvideBlockStateExtTy, RawBlock};
use rimecraft_chunk_palette::{
    IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw, Maybe, container::ProvidePalette,
};
use rimecraft_fluid::ProvideFluidStateExtTy;
use rimecraft_global_cx::{ProvideIdTy, ProvideNbtTy};
use rimecraft_registry::Registry;
use rimecraft_voxel_math::BlockPos;

use crate::{
    BlockEntityCell, Sealed,
    event::game_event,
    heightmap::{self, Heightmap},
    view::{
        HeightLimit,
        block::{
            BlockEntityView, BlockLuminanceView, BlockView, LockedBlockEntityViewMut,
            LockedBlockViewMut,
        },
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

/// Immutable chunk behaviors.
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
    fn highest_non_empty_section(&self) -> Option<usize> {
        self.sections().iter().rposition(|s| !s.lock().is_empty())
    }

    /// Peeks the heightmaps of this chunk.
    #[inline]
    fn peek_heightmaps<F, T>(&self, pk: F) -> T
    where
        F: for<'a> FnOnce(&'a AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>) -> T,
    {
        let rg = self.as_base_chunk().0.heightmaps.read();
        pk(&rg)
    }

    /// Peeks the heightmaps of this chunk.
    #[inline]
    fn peek_heightmaps_mut_locked<F, T>(&self, pk: F) -> T
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

    /// Peeks the [`game_event::Dispatcher`] of given Y section corrdinate.
    #[inline]
    fn peek_game_event_dispatcher<F, T>(&self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        let _ = y_section_coord;
        drop(f);
        None
    }

    /// Gets the [`game_event::Dispatcher`] of given Y section corrdinate.
    #[inline]
    fn game_event_dispatcher(
        &self,
        y_section_coord: i32,
    ) -> Option<Arc<game_event::Dispatcher<'w, Cx>>> {
        self.peek_game_event_dispatcher(y_section_coord, Arc::clone)
    }
}

/// Mutable chunk behaviors.
pub trait ChunkMut<'w, Cx>
where
    Self: AsBaseChunkMut<'w, Cx>
        + Chunk<'w, Cx>
        + LockedBlockViewMut<'w, Cx>
        + LockedBlockEntityViewMut<'w, Cx>,
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
    fn peek_heightmaps_mut<F, T>(&mut self, pk: F) -> T
    where
        F: for<'a> FnOnce(&'a mut AHashMap<Cx::HeightmapType, Heightmap<'w, Cx>>) -> T,
    {
        pk(self.as_base_chunk_mut().0.heightmaps.get_mut())
    }

    /// Returns the index of highest non-empty [`ChunkSection`] in this chunk.
    ///
    /// This method is the same as [`Chunk::highest_non_empty_section`] but lock-free.
    ///
    /// See [`ChunkSection::is_empty`].
    fn highest_non_empty_section_lf(&mut self) -> Option<usize> {
        self.sections_mut()
            .iter_mut()
            .rposition(|s| !s.get_mut().is_empty())
    }

    /// Peeks the [`game_event::Dispatcher`] of given Y section corrdinate.
    ///
    /// This method is the same as [`Chunk::peek_game_event_dispatcher`] but lock-free.
    #[inline]
    fn peek_game_event_dispatcher_lf<F, T>(&mut self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        let _ = y_section_coord;
        drop(f);
        None
    }

    /// Gets the [`game_event::Dispatcher`] of given Y section corrdinate.
    ///
    /// This method is the same as [`Chunk::game_event_dispatcher`] but lock-free.
    #[inline]
    fn game_event_dispatcher_lf(
        &mut self,
        y_section_coord: i32,
    ) -> Option<Arc<game_event::Dispatcher<'w, Cx>>> {
        self.peek_game_event_dispatcher_lf(y_section_coord, Arc::clone)
    }
}
