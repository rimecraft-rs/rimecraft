use std::{collections::HashMap, fmt::Debug, hash::Hash};

use fastnbt::IntArray;
use rimecraft_block::RawBlock;
use rimecraft_fluid::RawFluid;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{ProvideRegistry, Reg};
use rimecraft_voxel_math::direction::EightWayDirection;
use serde::Deserialize;

use crate::view::HeightLimit;

use super::ChunkTy;

/// Upgrade data for a chunk.
pub struct UpgradeData<'w, Cx>
where
    Cx: ChunkTy<'w>,
{
    sides_to_upgrade: Vec<EightWayDirection>,
    center_indices_upgrade: Box<[Box<[i32]>]>,

    block_ticks: Vec<TickedBlock<'w, Cx>>,
    fluid_ticks: Vec<TickedFluid<'w, Cx>>,
}

type TickedBlock<'w, Cx> = TickedReg<'w, RawBlock<'w, Cx>, <Cx as ProvideIdTy>::Id>;
type TickedFluid<'w, Cx> = TickedReg<'w, RawFluid<'w, Cx>, <Cx as ProvideIdTy>::Id>;

#[derive(Debug)]
struct TickedReg<'r, T, K>(Reg<'r, K, T>);

impl<'w, Cx> UpgradeData<'w, Cx>
where
    Cx: ChunkTy<'w>
        + ProvideRegistry<'w, Cx::Id, RawBlock<'w, Cx>>
        + ProvideRegistry<'w, Cx::Id, RawFluid<'w, Cx>>,
    Cx::Id: Hash + Eq,
{
    /// Creates a new upgrade data from given *serialized NBT data* and the height limit.
    ///
    /// The NBT data should be any `fastnbt` deserializer.
    ///
    /// # Errors
    ///
    /// This method can fail if the given NBT data is invalid.
    pub fn new<'de, D>(
        nbt: D,
        height_limit: HeightLimit,
    ) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
        Cx::Id: Deserialize<'de>,
    {
        #[derive(Deserialize)]
        #[serde(bound(deserialize = r#"
                Cx::Id: Deserialize<'de> + Hash + Eq,
                Cx: ProvideRegistry<'w, Cx::Id, RawBlock<'w, Cx>>
                    + ProvideRegistry<'w, Cx::Id, RawFluid<'w, Cx>>
                    + ChunkTy<'w>
                "#))]
        struct Serialized<'w, Cx>
        where
            Cx: ChunkTy<'w>,
        {
            #[serde(rename = "Indices")]
            #[serde(default)]
            indices: HashMap<String, IntArray>,

            #[serde(rename = "Sides")]
            #[serde(default)]
            sides: i32,

            neighbor_block_ticks: Vec<TickedBlock<'w, Cx>>,
            neighbor_fluid_ticks: Vec<TickedFluid<'w, Cx>>,
        }

        let Serialized {
            indices,
            sides,
            neighbor_block_ticks: block_ticks,
            neighbor_fluid_ticks: fluid_ticks,
        } = Serialized::<'w, Cx>::deserialize(nbt)?;

        Ok(Self {
            sides_to_upgrade: {
                let mut dirs = Vec::with_capacity(EightWayDirection::COUNT);
                for dir in EightWayDirection::ALL {
                    if (sides & 1 << dir as u8) != 0 {
                        dirs.push(dir);
                    }
                }
                dirs.shrink_to_fit();
                dirs
            },
            center_indices_upgrade: {
                let len = height_limit.count_vertical_sections() as usize;
                let mut center_indices_upgrade: Box<[Box<[i32]>]> =
                    vec![vec![].into_boxed_slice(); len].into_boxed_slice();
                for (section, indices) in indices
                    .iter()
                    .filter_map(|(k, v)| k.parse::<usize>().ok().map(|k| (k, v)))
                    .filter(|(k, _)| *k < len)
                {
                    center_indices_upgrade[section] = indices.iter().copied().collect();
                }
                center_indices_upgrade
            },
            block_ticks,
            fluid_ticks,
        })
    }
}

impl<'w, Cx> Debug for UpgradeData<'w, Cx>
where
    Cx: ChunkTy<'w> + Debug,
    Cx::Id: Debug,
    Cx::BlockStateExt: Debug,
    Cx::BlockStateList: Debug,
    Cx::FluidStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpgradeData")
            .field("sides_to_upgrade", &self.sides_to_upgrade)
            .field("block_ticks", &self.block_ticks)
            .field("fluid_ticks", &self.fluid_ticks)
            .field("center_indices_upgrade", &self.center_indices_upgrade)
            .finish()
    }
}

mod _serde {
    use rimecraft_registry::entry::RefEntry;
    use serde::Serialize;

    use super::*;

    impl<T, K> Serialize for TickedReg<'_, T, K>
    where
        K: Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let e: &RefEntry<_, _> = self.0.as_ref();
            e.key().value().serialize(serializer)
        }
    }

    impl<'a, 'de, K, T> Deserialize<'de> for TickedReg<'a, T, K>
    where
        T: ProvideRegistry<'a, K, T> + 'a,
        K: Deserialize<'de> + Hash + Eq + 'a,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let key = K::deserialize(deserializer)?;
            Ok(Self(T::registry().get(&key).unwrap_or_default()))
        }
    }
}
