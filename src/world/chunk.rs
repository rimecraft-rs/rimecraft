use std::{ops::Deref, sync::atomic};

use crate::{block, prelude::*, util::math::ChunkPos};

use super::{palette, HeightLimitView};

pub struct RawChunk<W: HeightLimitView> {
    pub pos: ChunkPos,
    pub height_limit_view: std::sync::Arc<W>,
    pub upgrade_data: UpgradeData,
}

impl<W: HeightLimitView> RawChunk<W> {}

pub struct ChunkSection {
    non_empty_block_count: atomic::AtomicU16,
    random_tickable_block_count: atomic::AtomicU16,
    non_empty_fluid_count: atomic::AtomicU16,
    bs_container: palette::Container<'static, block::SharedBlockState>,
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

    pub fn new(nbt: &crate::nbt::NbtCompound, world: &impl HeightLimitView) -> Self {
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
                        .get_from_id(&Identifier::try_parse(id).ok()?)
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
                        .get_from_id(&Identifier::try_parse(id).ok()?)
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
