use std::{ops::Deref, sync::atomic};

use crate::{block, fluid, prelude::*, util::math::ChunkPos};

use super::{biome, palette};

pub trait Chunk<'w>: super::BlockView + super::LightSourceView + std::any::Any {
    fn pos(&self) -> ChunkPos;

    fn sections(&self) -> &[Option<Section<'w>>];
    fn sections_mut(&self) -> &mut [Option<Section<'w>>];

    fn heightmaps(
        &self,
    ) -> &[(
        crate::Ref<'static, super::heightmap::Type>,
        super::heightmap::Heightmap,
    )];

    fn heightmaps_mut(
        &self,
    ) -> &mut [(
        crate::Ref<'static, super::heightmap::Type>,
        super::heightmap::Heightmap,
    )];

    fn set_block_state(
        &self,
        pos: BlockPos,
        state: block::SharedBlockState,
        moved: bool,
    ) -> Option<block::SharedBlockState>;

    fn set_block_entity(&self, be: block::entity::BlockEntity);
    fn remove_block_entity(&self, pos: BlockPos);
}

pub struct WorldChunk<'w> {
    sections: Vec<Section<'w>>,
    height_limit_view: *const dyn super::HeightLimitView,
}

mod chunk_imp {
    pub fn fill_sections<'w>(
        sections: &mut [Option<super::Section<'w>>],
        registry: &'w super::biome::SharedRegistry,
    ) {
        for value in sections.iter_mut() {
            if value.is_none() {
                *value = Some(super::Section::from_biome_registry(registry))
            }
        }
    }
}

pub struct Section<'w> {
    pub biome_container: palette::Container<'w, biome::Shared<'w>>,
    pub block_state_container: palette::Container<'static, block::SharedBlockState>,

    lock: std::sync::Mutex<()>,

    non_empty_block_count: atomic::AtomicU16,
    non_empty_fluid_count: atomic::AtomicU16,
    random_tickable_block_count: atomic::AtomicU16,
}

impl<'w> Section<'w> {
    pub fn from_biome_registry(biomes: &'w super::biome::SharedRegistry) -> Self {
        Self {
            block_state_container: palette::Container::from_initialize(
                block::STATE_IDS.deref().deref(),
                block::Block::default().default_state(),
                palette::Provider::BlockState,
            ),
            biome_container: palette::Container::from_initialize(
                biomes,
                todo!(),
                palette::Provider::Biome,
            ),
            lock: std::sync::Mutex::new(()),
            non_empty_block_count: atomic::AtomicU16::new(0),
            non_empty_fluid_count: atomic::AtomicU16::new(0),
            random_tickable_block_count: atomic::AtomicU16::new(0),
        }
    }

    pub fn get_block_state(&self, x: i32, y: i32, z: i32) -> Option<block::SharedBlockState> {
        self.block_state_container.get((x, y, z))
    }

    pub fn set_block_state(
        &self,
        pos: (i32, i32, i32),
        state: block::SharedBlockState,
    ) -> block::SharedBlockState {
        let _lock = self.lock.lock();
        unsafe { self.set_block_state_unchecked(pos, state) }
    }

    pub unsafe fn set_block_state_unchecked(
        &self,
        pos: (i32, i32, i32),
        state: block::SharedBlockState,
    ) -> block::SharedBlockState {
        let bs = unsafe { self.block_state_container.swap_unchecked(pos, state) };
        let fs = bs.fluid_state();
        let fs2 = state.fluid_state();

        if !block::EVENTS.read().is_air(bs.deref()) {
            let nebc = self.non_empty_block_count.load(atomic::Ordering::Acquire);
            self.non_empty_block_count
                .store(nebc - 1, atomic::Ordering::Release);
        }

        if !fluid::EVENTS.read().is_empty(fs.deref()) {
            let nefc = self.non_empty_fluid_count.load(atomic::Ordering::Acquire);
            self.non_empty_fluid_count
                .store(nefc - 1, atomic::Ordering::Release);
        }

        if !block::EVENTS.read().is_air(state.deref()) {
            let nebc = self.non_empty_block_count.load(atomic::Ordering::Acquire);
            self.non_empty_block_count
                .store(nebc + 1, atomic::Ordering::Release);

            if block::EVENTS.read().has_random_ticks(state.deref()) {
                let rtbc = self
                    .random_tickable_block_count
                    .load(atomic::Ordering::Acquire);
                self.random_tickable_block_count
                    .store(rtbc + 1, atomic::Ordering::Release);
            }
        }

        if !fluid::EVENTS.read().is_empty(fs2.deref()) {
            let nefc = self.non_empty_fluid_count.load(atomic::Ordering::Acquire);
            self.non_empty_fluid_count
                .store(nefc + 1, atomic::Ordering::Release);
        }

        bs
    }

    pub fn is_empty(&self) -> bool {
        self.non_empty_block_count.load(atomic::Ordering::Acquire) == 0
    }

    pub fn has_random_ticks(&self) -> bool {
        self.has_random_block_ticks() || self.has_random_fluid_ticks()
    }

    pub fn has_random_block_ticks(&self) -> bool {
        self.random_tickable_block_count
            .load(atomic::Ordering::Acquire)
            > 0
    }

    pub fn has_random_fluid_ticks(&self) -> bool {
        self.non_empty_fluid_count.load(atomic::Ordering::Acquire) > 0
    }

    pub fn calculate_counts(&self) {
        let mut counter = BlockStateCounter::default();
        self.block_state_container.count(&mut counter);

        self.non_empty_block_count
            .store(counter.non_empty_block_count, atomic::Ordering::Release);
        self.random_tickable_block_count.store(
            counter.random_tickable_block_count,
            atomic::Ordering::Release,
        );
        self.non_empty_fluid_count
            .store(counter.non_empty_fluid_count, atomic::Ordering::Release);
    }
}

#[derive(Default)]
struct BlockStateCounter {
    non_empty_block_count: u16,
    non_empty_fluid_count: u16,
    random_tickable_block_count: u16,
}

impl BlockStateCounter {}

impl palette::ContainerCounter<block::SharedBlockState> for BlockStateCounter {
    fn accept(&mut self, value: block::SharedBlockState, count: usize) {
        let fs = value.fluid_state();

        if !block::EVENTS.read().is_air(value.deref()) {
            self.non_empty_block_count += count as u16;

            if block::EVENTS.read().has_random_ticks(value.deref()) {
                self.random_tickable_block_count += count as u16;
            }
        }

        if !fluid::EVENTS.read().is_empty(fs.deref()) {
            self.non_empty_block_count += count as u16;

            if fluid::EVENTS.read().has_random_ticks(fs.deref()) {
                self.non_empty_fluid_count += count as u16;
            }
        }
    }
}

pub struct UpgradeData {
    sides_to_upgrade: Vec<crate::util::math::EightWayDirection>,
    block_ticks: Vec<super::tick::Tick<crate::block::Block>>,
    fluid_ticks: Vec<super::tick::Tick<crate::fluid::Fluid>>,
    /// The `0` (usize) field is for the outer vector's lenth.
    center_indices_to_upgrade: (usize, Vec<Vec<i32>>),
}

impl UpgradeData {
    const INDICES_KEY: &str = "Indices";

    pub fn new(nbt: &crate::nbt::NbtCompound, world: &impl super::HeightLimitView) -> Self {
        let mut this = Self {
            sides_to_upgrade: Vec::new(),
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            center_indices_to_upgrade: (world.count_vertical_sections() as usize, Vec::new()),
        };

        if let Some(compound) = nbt.get_compound(Self::INDICES_KEY) {
            for i in 0..this.center_indices_to_upgrade.0 {
                let string = i.to_string();
                this.center_indices_to_upgrade.1.push(
                    if let Some(slice) = compound.get_i32_slice(&string) {
                        slice.to_vec()
                    } else {
                        Vec::new()
                    },
                )
            }
        }

        let j = nbt.get_i32("Sides").unwrap_or_default();

        for ewd in crate::util::math::EightWayDirection::values() {
            if (j & 1 << ewd as u8) != 0 {
                this.sides_to_upgrade.push(ewd);
            }
        }

        Self::add_neighbor_ticks(
            nbt,
            "neighbor_block_ticks",
            |id| {
                Some(
                    crate::registry::BLOCK
                        .get_from_id(&Id::try_parse(id).ok()?)
                        .map(|e| e.1.deref().clone())
                        .unwrap_or_default(),
                )
            },
            &mut this.block_ticks,
        );

        Self::add_neighbor_ticks(
            nbt,
            "neighbor_fluid_ticks",
            |id| {
                Some(
                    crate::registry::FLUID
                        .get_from_id(&Id::try_parse(id).ok()?)
                        .map(|e| e.1.deref().clone())
                        .unwrap_or_default(),
                )
            },
            &mut this.fluid_ticks,
        );

        this
    }

    fn add_neighbor_ticks<T, F>(
        nbt: &crate::nbt::NbtCompound,
        key: &str,
        name_to_type: F,
        ticks: &mut Vec<super::tick::Tick<T>>,
    ) where
        F: Fn(&str) -> Option<T>,
    {
        if let Some(list) = nbt.get_slice(key) {
            for value in list.iter() {
                if let Some(tick) = super::tick::Tick::from_nbt(
                    match value {
                        crate::nbt::NbtElement::Compound(c) => c,
                        _ => continue,
                    },
                    |n| name_to_type(n),
                ) {
                    ticks.push(tick)
                }
            }
        }
    }
}
