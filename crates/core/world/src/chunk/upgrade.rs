use std::{cell::Cell, fmt::Debug, hash::Hash, marker::PhantomData};

use local_cx::{LocalContext, LocalContextExt as _, serde::DeserializeWithCx};
use rimecraft_block::RawBlock;
use rimecraft_fluid::RawFluid;
use rimecraft_global_cx::{ProvideIdTy, ProvideNbtTy};
use rimecraft_registry::{Reg, Registry};
use rimecraft_voxel_math::direction::EightWayDirection;
use serde::{
    Deserialize,
    de::{DeserializeOwned, DeserializeSeed},
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
    pub fn new<'de, D, Local>(
        nbt: D,
        height_limit: HeightLimit,
        cx: Local,
    ) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
        Cx::Id: Deserialize<'de>,
        Cx::IntArray: DeserializeOwned,
        Local: LocalContext<&'w Registry<Cx::Id, RawBlock<'w, Cx>>>
            + LocalContext<&'w Registry<Cx::Id, RawFluid<'w, Cx>>>,
    {
        let indices_len = height_limit.count_vertical_sections() as usize;

        thread_local! {
            static LENGTH: Cell<usize> = const { Cell::new(0) };
        }

        LENGTH.set(indices_len);

        struct Serialized<'w, Cx>
        where
            Cx: ChunkCx<'w>,
        {
            data: UpgradeData<'w, Cx>,
        }

        impl<'w, 'de, Cx, L> DeserializeWithCx<'de, L> for Serialized<'w, Cx>
        where
            Cx: ChunkCx<'w, IntArray: DeserializeOwned, Id: Deserialize<'de>>,
            L: LocalContext<&'w Registry<Cx::Id, RawBlock<'w, Cx>>>
                + LocalContext<&'w Registry<Cx::Id, RawFluid<'w, Cx>>>,
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
                    Cx: ChunkCx<'w, IntArray: DeserializeOwned, Id: Deserialize<'de>>,
                    L: LocalContext<&'w Registry<Cx::Id, RawBlock<'w, Cx>>>
                        + LocalContext<&'w Registry<Cx::Id, RawFluid<'w, Cx>>>,
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
                        let mut sides_key = 0usize;

                        let mut block_ticks: Vec<TickedBlock<'w, Cx>> = vec![];
                        let mut fluid_ticks: Vec<TickedFluid<'w, Cx>> = vec![];

                        while let Some(field) = map.next_key::<Field>()? {
                            match field {
                                Field::Indices => map.next_value_seed(IndicesSeed(
                                    &mut indices,
                                    PhantomData::<Cx>,
                                ))?,
                                Field::Sides => sides_key = map.next_value::<u32>()? as usize,
                                Field::NBlockTicks => {
                                    map.next_value_seed(TicksSeed::<Cx, RawBlock<'w, Cx>, L>(
                                        &mut block_ticks,
                                        self.0,
                                    ))?
                                }
                                Field::NFluidTicks => {
                                    map.next_value_seed(TicksSeed::<Cx, RawFluid<'w, Cx>, L>(
                                        &mut fluid_ticks,
                                        self.0,
                                    ))?
                                }
                                Field::Other => {}
                            }
                        }

                        let sides: Vec<_> = EightWayDirection::ALL
                            .into_iter()
                            .filter(|dir| (sides_key & (1 << (*dir as u8))) != 0)
                            .collect();

                        Ok(Serialized {
                            data: UpgradeData {
                                sides_to_upgrade: sides,
                                center_indices_upgrade: indices,
                                block_ticks,
                                fluid_ticks,
                            },
                        })
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

                struct TicksSeed<'a, 'w, Cx, T, L>(&'a mut Vec<TickedReg<'w, T, Cx::Id>>, L)
                where
                    Cx: ChunkCx<'w>;

                impl<'de, 'w, Cx, T, L> serde::de::Visitor<'de> for TicksSeed<'_, 'w, Cx, T, L>
                where
                    Cx: ChunkCx<'w, Id: Deserialize<'de>>,
                    L: LocalContext<&'w Registry<Cx::Id, T>>,
                {
                    type Value = ();

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "a sequence containing ticked registries")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        if let Some(len) = seq.size_hint() {
                            self.0.reserve(len);
                        }
                        while let Some(reg) = seq.next_element_seed(
                            self.1.with(PhantomData::<TickedReg<'w, T, Cx::Id>>),
                        )? {
                            self.0.push(reg);
                        }
                        Ok(())
                    }
                }

                impl<'de, 'w, Cx, T, L> DeserializeSeed<'de> for TicksSeed<'_, 'w, Cx, T, L>
                where
                    Cx: ChunkCx<'w, Id: Deserialize<'de>>,
                    L: LocalContext<&'w Registry<Cx::Id, T>>,
                {
                    type Value = ();

                    #[inline]
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        deserializer.deserialize_seq(self)
                    }
                }

                let cx = deserializer.local_cx;
                deserializer
                    .inner
                    .deserialize_map(Visitor(cx, PhantomData::<&'w Cx>, LENGTH.get()))
            }
        }

        Serialized::deserialize_with_cx(cx.with(nbt)).map(|ser| ser.data)
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
    use local_cx::{LocalContext, serde::DeserializeWithCx};
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
            Reg::to_id(self.0).serialize(serializer)
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
