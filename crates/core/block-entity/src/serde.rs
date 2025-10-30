//! Serialization and deserialization of block entities.

use std::{fmt::Debug, marker::PhantomData};

use bitflags::bitflags;
use component::{RawErasedComponentType, map::ComponentMap};
use local_cx::{LocalContext, LocalContextExt as _, WithLocalCx, serde::SerializeWithCx};
use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_registry::Registry;
use rimecraft_voxel_math::BlockPos;
use serde::{Deserialize, Serialize, de::DeserializeSeed};
use serde_private::de::ContentVisitor;

use crate::{BlockEntity, BlockEntityCx, DynErasedRawBlockEntityType, RawBlockEntity};

bitflags! {
    /// Essential flags for serializing a block entity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        /// Serializes the registration identifier.
        const ID = 1u8 << 0;
        /// Serializes the position.
        const POS = 1u8 << 1;
        /// Serializes the component map.
        const COMPONENTS = 1u8 << 2;
    }
}

impl Flags {
    /// Serializes the identifier and position.
    #[inline]
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

/// Data flagged by [`Flags`], for serialization.
#[derive(Debug)]
pub struct Flagged<T>(pub T, pub Flags);

impl<'a, T, Cx> SerializeWithCx<Cx::LocalContext<'a>> for Flagged<&RawBlockEntity<'a, T, Cx>>
where
    Cx: BlockEntityCx<'a>,
    T: ?Sized + Serialize,
    Cx::Id: Serialize,
{
    fn serialize_with_cx<S>(
        &self,
        serializer: WithLocalCx<S, Cx::LocalContext<'a>>,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;
        let cx = serializer.local_cx;
        let mut map = serializer.inner.serialize_map(None)?;
        for flag in self.1.iter() {
            match flag {
                Flags::COMPONENTS => {
                    if !self.0.components.is_empty() {
                        map.serialize_entry(&Field::Components, &cx.with(&self.0.components))?
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
            .serialize(serde_private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl<'a, T, Cx> SerializeWithCx<Cx::LocalContext<'a>> for Flagged<RawBlockEntity<'a, T, Cx>>
where
    Cx: BlockEntityCx<'a>,
    T: Serialize,
    Cx::Id: Serialize,
{
    fn serialize_with_cx<S>(
        &self,
        serializer: WithLocalCx<S, Cx::LocalContext<'a>>,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Flagged(&self.0, self.1).serialize_with_cx(serializer)
    }
}

enum Field<'de> {
    Id,
    Components,

    X,
    Y,
    Z,

    Other(serde_private::de::Content<'de>),
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
                    other => Ok(Field::Other(serde_private::de::Content::String(
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
                    other => Ok(Field::Other(serde_private::de::Content::Str(other))),
                }
            }
        }

        deserializer.deserialize_identifier(Visitor(PhantomData))
    }
}

/// This serializes the block entity using default value of [`Flags`].
impl<'a, T, Cx> SerializeWithCx<Cx::LocalContext<'a>> for RawBlockEntity<'a, T, Cx>
where
    Cx: BlockEntityCx<'a>,
    T: ?Sized + Serialize,
    Cx::Id: Serialize,
{
    fn serialize_with_cx<S>(
        &self,
        serializer: WithLocalCx<S, Cx::LocalContext<'a>>,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Flagged(self, Flags::default()).serialize_with_cx(serializer)
    }
}

/// Seed for deserializing a block state.
pub struct Seed<'a, Cx, Local>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Position of the block state.
    pub pos: BlockPos,
    /// State of the block.
    pub state: BlockState<'a, Cx>,

    /// The local context.
    pub local_cx: Local,
}

impl<'a, 'de, Cx> DeserializeSeed<'de> for Seed<'a, Cx, Cx::LocalContext<'a>>
where
    Cx: BlockEntityCx<'a, Id: Deserialize<'de>>,
    Cx::LocalContext<'a>: LocalContext<&'a Registry<Cx::Id, RawErasedComponentType<'a, Cx>>>
        + LocalContext<&'a Registry<Cx::Id, DynErasedRawBlockEntityType<'a, Cx>>>,
{
    type Value = Box<BlockEntity<'a, Cx>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a, Cx, L>(BlockPos, BlockState<'a, Cx>, L)
        where
            Cx: ProvideBlockStateExtTy;

        impl<'a, 'de, Cx> serde::de::Visitor<'de> for Visitor<'a, Cx, Cx::LocalContext<'a>>
        where
            Cx: BlockEntityCx<'a, Id: Deserialize<'de>>,
            Cx::LocalContext<'a>: LocalContext<&'a Registry<Cx::Id, RawErasedComponentType<'a, Cx>>>
                + LocalContext<&'a Registry<Cx::Id, DynErasedRawBlockEntityType<'a, Cx>>>,
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
                use serde_private::de::Content;
                let mut collect: Vec<Option<(Content<'de>, Content<'de>)>> =
                    Vec::with_capacity(map.size_hint().map_or(0, |i| i - 1));

                while let Some(field) = map.next_key::<Field<'de>>()? {
                    match field {
                        Field::Id => id = Some(map.next_value()?),
                        Field::Components => {
                            components = Some(map.next_value_seed(WithLocalCx {
                                inner: PhantomData,
                                local_cx: self.2,
                            })?)
                        }
                        // Skip position information
                        Field::X | Field::Y | Field::Z => {}
                        Field::Other(c) => {
                            collect.push(Some((c, map.next_value_seed(ContentVisitor::new())?)))
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let components = components.unwrap_or(ComponentMap::EMPTY);

                let ty =
                    std::convert::identity::<&Registry<_, DynErasedRawBlockEntityType<'_, _>>>(
                        self.2.acquire(),
                    )
                    .get(&id)
                    .ok_or_else(|| {
                        serde::de::Error::custom(format!("unknown block entity type {id}"))
                    })?;
                let mut be = ty
                    .erased_instantiate(self.0, self.1, ty)
                    .ok_or_else(|| serde::de::Error::custom("failed to create block entity"))?;
                rimecraft_serde_update::Update::update(
                    &mut *be,
                    serde_private::de::FlatMapDeserializer(&mut collect, PhantomData),
                )?;
                be.components = components;

                Ok(be)
            }
        }

        deserializer.deserialize_map(Visitor(self.pos, self.state, self.local_cx))
    }
}

impl<'a, 'de, T, Cx> rimecraft_serde_update::Update<'de> for RawBlockEntity<'a, T, Cx>
where
    Cx: BlockEntityCx<'a>,
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

impl<'a, Cx, L> Debug for Seed<'a, Cx, L>
where
    Cx: BlockEntityCx<'a, Id: Debug, BlockStateExt: Debug> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeserializeSeed")
            .field("pos", &self.pos)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}
