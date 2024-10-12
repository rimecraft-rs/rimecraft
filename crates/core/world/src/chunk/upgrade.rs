use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use local_cx::serde::DeserializeWithCx;
use rimecraft_block::RawBlock;
use rimecraft_fluid::RawFluid;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{ProvideRegistry, Reg};
use rimecraft_voxel_math::direction::EightWayDirection;
use serde::Deserialize;

use crate::view::HeightLimit;

use super::ChunkCx;

/// Upgrade data for a chunk.
pub struct UpgradeData<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    sides_to_upgrade: Vec<EightWayDirection>,
    center_indices_upgrade: Box<[Box<[i32]>]>,

    block_ticks: Vec<TickedBlock<'w, Cx>>,
    fluid_ticks: Vec<TickedFluid<'w, Cx>>,
}

type TickedBlock<'w, Cx> = TickedReg<'w, RawBlock<'w, Cx>, <Cx as ProvideIdTy>::Id>;
type TickedFluid<'w, Cx> = TickedReg<'w, RawFluid<'w, Cx>, <Cx as ProvideIdTy>::Id>;

#[derive(Debug)]
#[repr(transparent)]
struct TickedReg<'r, T, K>(Reg<'r, K, T>);

impl<'w, Cx> UpgradeData<'w, Cx>
where
    Cx: ChunkCx<'w>,
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
        Cx::IntArray: Deserialize<'de>,
    {
        struct Serialized<'w, Cx>
        where
            Cx: ChunkCx<'w>,
        {
            // `Indices`, defaulted.
            indices: HashMap<String, Cx::IntArray>,

            // `Sides`, defaulted.
            sides: i32,

            neighbor_block_ticks: Vec<TickedBlock<'w, Cx>>,
            neighbor_fluid_ticks: Vec<TickedFluid<'w, Cx>>,
        }

        impl<'w, 'de, Cx, L> DeserializeWithCx<'de, L> for Serialized<'w, Cx>
        where
            Cx: ChunkCx<'w>,
        {
            fn deserialize_with_cx<D>(
                deserializer: local_cx::WithLocalCx<D, L>,
            ) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                enum Field {
                    Indices,
                    Sides,

                    NBlockTicks,
                    NFluidTicks,
                }

                struct FieldVisitor;

                impl serde::de::Visitor<'_> for FieldVisitor {
                    type Value = Field;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "a field identifier")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(match v {
                            "Indices" => Field::Indices,
                            "Sides" => Field::Sides,
                            "neighbor_block_ticks" => Field::NBlockTicks,
                            "neighbor_fluid_ticks" => Field::NFluidTicks,
                            _ => {
                                return Err(serde::de::Error::unknown_field(
                                    v,
                                    &[
                                        "Indices",
                                        "Sides",
                                        "neighbor_block_ticks",
                                        "neighbor_fluid_ticks",
                                    ],
                                ))
                            }
                        })
                    }
                }

                impl<'de> Deserialize<'de> for Field {
                    #[inline]
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        deserializer.deserialize_identifier(FieldVisitor)
                    }
                }

                struct Visitor<'w, L, Cx>(L, PhantomData<&'w Cx>);

                impl<'w, 'de, L, Cx> serde::de::Visitor<'de> for Visitor<'w, L, Cx>
                where
                    Cx: ChunkCx<'w>,
                {
                    type Value = Serialized<'w, Cx>;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(
                            formatter,
                            "a structure containing chunk upgrade data information"
                        )
                    }

                    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>,
                    {
                        todo!()
                    }
                }

                struct IndicesVisitori {}

                let cx = deserializer.local_cx;
                todo!()
            }
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
                    .into_iter()
                    .filter_map(|(k, v)| k.parse::<usize>().ok().map(|k| (k, v)))
                    .filter(|(k, _)| *k < len)
                {
                    center_indices_upgrade[section] = indices.into();
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
    Cx: ChunkCx<'w> + Debug,
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
    use local_cx::{serde::DeserializeWithCx, LocalContext};
    use rimecraft_registry::{entry::RefEntry, Registry};
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
            Reg::to_id(self.0);
        }
    }

    impl<'a, 'de, K, T, L> DeserializeWithCx<'de, L> for TickedReg<'a, T, K>
    where
        K: Deserialize<'de> + Hash + Eq + 'a,
        L: LocalContext<&'a Registry<K, T>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, L>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let cx = deserializer.local_cx;
            let key = K::deserialize(deserializer.inner)?;
            let registry = cx.acquire();
            registry
                .get(&key)
                .or_else(|| registry.default_entry())
                .ok_or_else(|| serde::de::Error::custom("no valid entry deserialized"))
                .map(Self)
        }
    }
}
