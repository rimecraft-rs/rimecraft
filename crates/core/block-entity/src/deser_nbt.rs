//! Helper module for deserializing NBT data into block entities.

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_global_cx::ProvideNbtTy;
use rimecraft_registry::ProvideRegistry;
use rimecraft_voxel_math::BlockPos;
use serde::{de::DeserializeSeed, Deserialize, Deserializer};

use crate::{BlockEntity, ProvideBlockEntity, RawBlockEntityTypeDyn};

/// The dummy value for block entity types.
pub const DUMMY: &str = "DUMMY";

/// A [`DeserializeSeed`] for [`BlockEntity`].
///
/// This deserializes the id of the block entity type and its data.
pub struct CreateFromNbt<'w, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// The position of the block entity.
    pub pos: BlockPos,
    /// The state of the [`Block`] the block entity belongs to.
    pub state: BlockState<'w, Cx>,

    /// Whether to respect the [`DUMMY`] value.
    pub respect_dummy: bool,
}

impl<Cx> Debug for CreateFromNbt<'_, Cx>
where
    Cx: ProvideBlockStateExtTy + Debug,
    Cx::BlockStateExt: Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateFromNbt")
            .field("pos", &self.pos)
            .field("state", &self.state)
            .finish()
    }
}

impl<'w, 'de, Cx> DeserializeSeed<'de> for CreateFromNbt<'w, Cx>
where
    Cx: ProvideBlockStateExtTy
        + ProvideRegistry<'w, Cx::Id, RawBlockEntityTypeDyn<'w, Cx>>
        + ProvideNbtTy,
    Cx::Id: Deserialize<'de> + Hash + Eq,
    Cx::BlockStateExt: ProvideBlockEntity<'w, Cx>,
{
    type Value = Option<Box<BlockEntity<'w, Cx>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<'w, Cx>(CreateFromNbt<'w, Cx>)
        where
            Cx: ProvideBlockStateExtTy;

        impl<'de, 'w, Cx> serde::de::Visitor<'de> for Visitor<'w, Cx>
        where
            Cx: ProvideBlockStateExtTy
                + ProvideRegistry<'w, Cx::Id, RawBlockEntityTypeDyn<'w, Cx>>
                + ProvideNbtTy,
            Cx::Id: Deserialize<'de> + Hash + Eq,
            Cx::BlockStateExt: ProvideBlockEntity<'w, Cx>,
        {
            type Value = Option<Box<BlockEntity<'w, Cx>>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a block entity")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                use serde::__private::de::Content;

                let mut id: Option<Cx::Id> = None;
                let mut is_dummy = false;
                let mut collect: Vec<Option<(Content<'de>, Content<'de>)>> =
                    Vec::with_capacity(map.size_hint().map_or(0, |i| i - 1));

                enum Field<'de> {
                    Id,
                    Other(Content<'de>),
                }

                impl<'de> Deserialize<'de> for Field<'de> {
                    fn deserialize<D>(deserializer: D) -> Result<Field<'de>, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        struct FieldVisitor;

                        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                            type Value = Field<'de>;

                            fn expecting(
                                &self,
                                formatter: &mut std::fmt::Formatter<'_>,
                            ) -> std::fmt::Result {
                                formatter.write_str("a field")
                            }

                            fn visit_str<E>(self, v: &str) -> Result<Field<'de>, E>
                            where
                                E: serde::de::Error,
                            {
                                match v {
                                    "id" => Ok(Field::Id),
                                    _ => Ok(Field::Other(Content::String(v.into()))),
                                }
                            }
                        }

                        deserializer.deserialize_identifier(FieldVisitor)
                    }
                }

                #[derive(Deserialize)]
                #[serde(untagged)]
                enum MaybeDummy<T> {
                    Dummy(Dummy),
                    Value(T),
                }

                pub struct Dummy;

                impl<'de> Deserialize<'de> for Dummy {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        struct DummyVisitor;

                        impl serde::de::Visitor<'_> for DummyVisitor {
                            type Value = Dummy;

                            fn expecting(
                                &self,
                                f: &mut std::fmt::Formatter<'_>,
                            ) -> std::fmt::Result {
                                write!(f, "a '{}' value", DUMMY)
                            }

                            #[inline]
                            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                            where
                                E: serde::de::Error,
                            {
                                if v == DUMMY {
                                    Ok(Dummy)
                                } else {
                                    Err(serde::de::Error::custom("expected 'DUMMY'"))
                                }
                            }
                        }

                        deserializer.deserialize_str(DummyVisitor)
                    }
                }

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }

                            if self.0.respect_dummy {
                                let dummy: MaybeDummy<Cx::Id> = map.next_value()?;
                                match dummy {
                                    MaybeDummy::Dummy(Dummy) => {
                                        is_dummy = true;
                                    }
                                    MaybeDummy::Value(i) => {
                                        id = Some(i);
                                    }
                                }
                            } else {
                                id = Some(map.next_value()?);
                            }
                        }
                        Field::Other(content) => {
                            collect.push(Some((content, map.next_value()?)));
                        }
                    }
                }

                if is_dummy {
                    let state = self.0.state.state;
                    let res = if let Some(constructor) =
                        ProvideBlockEntity::block_entity_constructor(state.data())
                    {
                        Ok(Some(constructor(self.0.pos)))
                    } else {
                        Ok(None)
                    };
                    res
                } else {
                    let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                    let registry: &rimecraft_registry::Registry<_, RawBlockEntityTypeDyn<'w, Cx>> =
                        Cx::registry();
                    let ty = registry.get(&id).ok_or_else(|| {
                        serde::de::Error::custom(format!("unknown block entity type: {}", id))
                    })?;

                    let mut be = ty.instantiate(self.0.pos, self.0.state);
                    // Update the block entity data.
                    if let Some(be) = &mut be {
                        rimecraft_serde_update::Update::update(
                            &mut be.data,
                            serde::__private::de::FlatMapDeserializer(&mut collect, PhantomData),
                        )?;
                    }
                    Ok(be)
                }
            }
        }

        deserializer.deserialize_map(Visitor(self))
    }
}
