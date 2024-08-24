//! Serialization and deserialization of block entities.

use std::{fmt::Debug, marker::PhantomData};

use bitflags::bitflags;
use component::{map::ComponentMap, RawErasedComponentType};
use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_registry::ProvideRegistry;
use rimecraft_voxel_math::BlockPos;
use serde::{de::DeserializeSeed, Deserialize, Serialize};

use crate::{BlockEntity, DynRawBlockEntityType, RawBlockEntity};

bitflags! {
    /// Essential flags for serializing a block entity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        /// Serializes the registration identifier.
        const ID = 0b00000001;
        /// Serializes the position.
        const POS = 0b00000010;
        /// Serializes the component map.
        const COMPONENTS = 0b00000100;
    }
}

impl Flags {
    /// Serializes the identifier and position.
    #[inline(always)]
    pub fn identifying_data() -> Self {
        Self::POS | Self::ID
    }
}

impl Default for Flags {
    #[inline]
    fn default() -> Self {
        Self::COMPONENTS
    }
}

/// Data flagged by [`SerializeFlags`], for serialization.
#[derive(Debug)]
pub struct Flagged<T>(pub T, pub Flags);

impl<T, Cx> Serialize for Flagged<&RawBlockEntity<'_, T, Cx>>
where
    Cx: ProvideBlockStateExtTy,
    T: ?Sized + Serialize,
    Cx::Id: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        for flag in self.1.iter() {
            match flag {
                Flags::COMPONENTS => {
                    if !self.0.components.is_empty() {
                        map.serialize_entry(&Field::Components, &self.0.components)?
                    }
                }
                Flags::ID => {
                    map.serialize_entry(&Field::Id, &self.0.ty)?;
                }
                Flags::POS => {
                    map.serialize_entry(&Field::X, &self.0.pos.x())?;
                    map.serialize_entry(&Field::Y, &self.0.pos.y())?;
                    map.serialize_entry(&Field::Z, &self.0.pos.z())?;
                }
                _ => {}
            }
        }
        self.0
            .data
            .serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl<T, Cx> Serialize for Flagged<RawBlockEntity<'_, T, Cx>>
where
    Cx: ProvideBlockStateExtTy,
    T: Serialize,
    Cx::Id: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Flagged(&self.0, self.1).serialize(serializer)
    }
}

enum Field<'de> {
    Id,
    Components,

    X,
    Y,
    Z,

    Other(serde::__private::de::Content<'de>),
}

impl Serialize for Field<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Field::Id => "id",
            Field::Components => "components",
            Field::X => "x",
            Field::Y => "y",
            Field::Z => "z",
            Field::Other(_) => unimplemented!(),
        })
    }
}

impl<'de> Deserialize<'de> for Field<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de>(PhantomData<&'de ()>);

        impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
            type Value = Field<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a field")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "id" => Ok(Field::Id),
                    "components" => Ok(Field::Components),
                    "x" => Ok(Field::X),
                    "y" => Ok(Field::Y),
                    "z" => Ok(Field::Z),
                    other => Ok(Field::Other(serde::__private::de::Content::String(
                        other.to_owned(),
                    ))),
                }
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "id" => Ok(Field::Id),
                    "components" => Ok(Field::Components),
                    "x" => Ok(Field::X),
                    "y" => Ok(Field::Y),
                    "z" => Ok(Field::Z),
                    other => Ok(Field::Other(serde::__private::de::Content::Str(other))),
                }
            }
        }

        deserializer.deserialize_identifier(Visitor(PhantomData))
    }
}

/// This serializes the block entity using default value of [`SerializeFlags`].
impl<T, Cx> Serialize for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
    T: ?Sized + Serialize,
    Cx::Id: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Flagged(self, Flags::default()).serialize(serializer)
    }
}

/// Seed for deserializing a block state.
pub struct Seed<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Position of the block state.
    pub pos: BlockPos,
    /// State of the block.
    pub state: BlockState<'a, Cx>,
}

impl<'a, 'de, Cx> DeserializeSeed<'de> for Seed<'a, Cx>
where
    Cx: ProvideBlockStateExtTy<Id: Deserialize<'de>>
        + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<'a, Cx>>
        + ProvideRegistry<'a, Cx::Id, DynRawBlockEntityType<'a, Cx>>,
{
    type Value = Box<BlockEntity<'a, Cx>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a, Cx>(BlockPos, BlockState<'a, Cx>)
        where
            Cx: ProvideBlockStateExtTy;

        impl<'a, 'de, Cx> serde::de::Visitor<'de> for Visitor<'a, Cx>
        where
            Cx: ProvideBlockStateExtTy<Id: Deserialize<'de>>
                + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<'a, Cx>>
                + ProvideRegistry<'a, Cx::Id, DynRawBlockEntityType<'a, Cx>>,
        {
            type Value = Box<BlockEntity<'a, Cx>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a block entity")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id: Option<Cx::Id> = None;
                let mut components: Option<ComponentMap<'a, Cx>> = None;
                use serde::__private::de::Content;
                let mut collect: Vec<Option<(Content<'de>, Content<'de>)>> =
                    Vec::with_capacity(map.size_hint().map_or(0, |i| i - 1));

                while let Some(field) = map.next_key::<Field<'de>>()? {
                    match field {
                        Field::Id => id = Some(map.next_value()?),
                        Field::Components => components = Some(map.next_value()?),
                        // Skip position information
                        Field::X | Field::Y | Field::Z => {}
                        Field::Other(c) => collect.push(Some((c, map.next_value()?))),
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let components = components.unwrap_or_else(|| ComponentMap::EMPTY);

                let ty = <Cx as ProvideRegistry<'_, _, DynRawBlockEntityType<'_, _>>>::registry()
                    .get(&id)
                    .ok_or_else(|| {
                        serde::de::Error::custom(format!("unknown block entity type {}", id))
                    })?;
                let mut be = ty
                    .instantiate(self.0, self.1)
                    .ok_or_else(|| serde::de::Error::custom("failed to create block entity"))?;
                rimecraft_serde_update::Update::update(
                    &mut *be,
                    serde::__private::de::FlatMapDeserializer(&mut collect, PhantomData),
                )?;
                be.components = components;

                Ok(be)
            }
        }

        deserializer.deserialize_map(Visitor(self.pos, self.state.clone()))
    }
}

impl<'de, T, Cx> rimecraft_serde_update::Update<'de> for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
    T: ?Sized + rimecraft_serde_update::Update<'de>,
{
    #[inline]
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.data.update(deserializer)
    }
}

impl<Cx> Debug for Seed<'_, Cx>
where
    Cx: ProvideBlockStateExtTy<Id: Debug, BlockStateExt: Debug> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeserializeSeed")
            .field("pos", &self.pos)
            .field("state", &self.state)
            .finish()
    }
}
