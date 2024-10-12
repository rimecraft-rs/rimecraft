use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use local_cx::serde::DeserializeWithCx;
use rimecraft_block::RawBlock;
use rimecraft_fluid::RawFluid;
use rimecraft_global_cx::{ProvideIdTy, ProvideNbtTy};
use rimecraft_registry::Reg;
use rimecraft_voxel_math::direction::EightWayDirection;
use serde::{
    de::{DeserializeOwned, DeserializeSeed},
    Deserialize,
};

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
    /// The NBT data should be any NBT deserializer.
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
            data: UpgradeData<'w, Cx>,
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

                    Other,
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
                            _ => Field::Other,
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

                struct Visitor<'w, L, Cx>(L, PhantomData<&'w Cx>, usize);

                impl<'w, 'de, L, Cx> serde::de::Visitor<'de> for Visitor<'w, L, Cx>
                where
                    Cx: ChunkCx<'w, IntArray: DeserializeOwned>,
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

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>,
                    {
                        let mut indices =
                            vec![std::convert::identity::<Box<[i32]>>(Box::new([])); self.2]
                                .into_boxed_slice();
                        let mut sides = 0usize;

                        // let mut block_ticks = None;
                        // let mut fluid_ticks = None;

                        while let Some(field) = map.next_key::<Field>()? {
                            match field {
                                Field::Indices => map.next_value_seed(IndicesSeed(
                                    &mut indices,
                                    PhantomData::<Cx>,
                                ))?,
                                Field::Sides => sides = map.next_value::<u32>()? as usize,
                                Field::NBlockTicks => todo!(),
                                Field::NFluidTicks => todo!(),
                                Field::Other => {}
                            }
                        }

                        todo!()
                    }
                }

                struct IndexSeed;

                impl serde::de::Visitor<'_> for IndexSeed {
                    type Value = isize;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "a number")
                    }

                    #[inline]
                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        v.parse().map_err(serde::de::Error::custom)
                    }

                    #[inline]
                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(v as isize)
                    }

                    #[inline]
                    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(v as isize)
                    }

                    #[inline]
                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(v as isize)
                    }
                }

                impl<'de> DeserializeSeed<'de> for IndexSeed {
                    type Value = isize;

                    #[inline]
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        deserializer.deserialize_any(Self)
                    }
                }

                struct IndicesSeed<'a, Cx>(&'a mut [Box<[i32]>], PhantomData<Cx>);

                impl<'de, Cx> serde::de::Visitor<'de> for IndicesSeed<'_, Cx>
                where
                    Cx: ProvideNbtTy<IntArray: DeserializeOwned>,
                {
                    type Value = ();

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "indices")
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>,
                    {
                        while let Some(index) = map.next_key_seed(IndexSeed)? {
                            let index = index as usize;
                            if let Some(buf) = self.0.get_mut(index) {
                                let array: Cx::IntArray = map.next_value()?;
                                *buf = array.into();
                            }
                        }
                        Ok(())
                    }
                }

                impl<'de, Cx> DeserializeSeed<'de> for IndicesSeed<'_, Cx>
                where
                    Cx: ProvideNbtTy<IntArray: DeserializeOwned>,
                {
                    type Value = ();

                    #[inline]
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        deserializer.deserialize_map(self)
                    }
                }

                let cx = deserializer.local_cx;
                todo!()
            }
        }
        todo!()
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
    use rimecraft_registry::Registry;
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
