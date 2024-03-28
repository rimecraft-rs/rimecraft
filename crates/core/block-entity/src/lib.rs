//! Rimecraft block entity primitives.

use std::{any::TypeId, fmt::Debug, hash::Hash, marker::PhantomData};

use erased_serde::{serialize_trait_object, Serialize as ErasedSerialize};

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_global_cx::{ProvideIdTy, ProvideNbtTy};
use rimecraft_registry::{entry::RefEntry, ProvideRegistry, Reg, Registry};
use rimecraft_serde_update::{erased::ErasedUpdate, update_trait_object};
use rimecraft_voxel_math::BlockPos;
use serde::{de::DeserializeSeed, Deserialize, Deserializer, Serialize};

pub use rimecraft_downcast::ToStatic;

/// A type of [`BlockEntity`].
pub trait RawBlockEntityType<Cx>: Debug
where
    Cx: ProvideBlockStateExtTy,
{
    /// Whether the block entity supports the given state.
    fn supports(&self, state: &BlockState<'_, Cx>) -> bool;

    /// Creates a new instance of the block entity.
    fn instantiate<'w>(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
    ) -> Option<Box<BlockEntity<'w, Cx>>>;
}

/// A type of [`BlockEntity`] that can be used in a type erased context.
pub type RawBlockEntityTypeDyn<'r, Cx> = Box<dyn RawBlockEntityType<Cx> + Send + Sync + 'r>;

/// A type of [`BlockEntity`].
pub type BlockEntityType<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, RawBlockEntityTypeDyn<'r, Cx>>;

/// An object holding extra data about a block in a world.
pub struct RawBlockEntity<'a, T: ?Sized, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    ty: BlockEntityType<'a, Cx>,
    pos: BlockPos,
    removed: bool,
    cached_state: BlockState<'a, Cx>,

    data: T,
}

impl<'a, T, Cx> RawBlockEntity<'a, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Creates a new block entity.
    pub fn new(
        ty: BlockEntityType<'a, Cx>,
        pos: BlockPos,
        state: BlockState<'a, Cx>,
        data: T,
    ) -> Self {
        Self {
            ty,
            pos,
            removed: false,
            cached_state: state,
            data,
        }
    }

    /// Gets the immutable inner data of this block entity.
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Gets the mutable inner data of this block entity.
    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T, Cx> Debug for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy + Debug,
    Cx::BlockStateExt: Debug,
    Cx::Id: Debug,
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockEntity")
            .field("type", &<&RefEntry<_, _>>::from(self.ty).key().value())
            .field("pos", &self.pos)
            .field("removed", &self.removed)
            .field("cached_state", &self.cached_state)
            .field("data", &&self.data)
            .finish()
    }
}

/// Type erased block entity data.
pub trait ErasedData
where
    Self: ErasedSerialize + for<'de> ErasedUpdate<'de> + Send + Sync + Debug + sealed::Sealed,
{
    /// The [`TypeId`] of this data.
    fn type_id(&self) -> TypeId;
}

serialize_trait_object! { ErasedData }
update_trait_object! { ErasedData }

mod sealed {
    pub trait Sealed {}
}

impl<T> sealed::Sealed for T where T: ErasedSerialize + for<'de> ErasedUpdate<'de> + Send + Sync {}

impl<T> ErasedData for T
where
    T: ErasedSerialize + for<'de> ErasedUpdate<'de> + ToStatic + Debug + Send + Sync,
{
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<<Self as ToStatic>::StaticRepr>()
    }
}

/// A type-erased variant of [`RawBlockEntity`].
pub type BlockEntity<'w, Cx> = RawBlockEntity<'w, dyn ErasedData + 'w, Cx>;

impl<'w, Cx> BlockEntity<'w, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Downcasts this type erased block entity into block entity with a concrete data type.
    ///
    /// This function returns an immutable reference if the type matches.
    pub fn downcast_ref<T: ToStatic>(&self) -> Option<&RawBlockEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(&*(self as *const BlockEntity<'w, Cx> as *const RawBlockEntity<'w, T, Cx>))
            }
        } else {
            None
        }
    }

    /// Downcasts this type erased block entity into block entity with a concrete data type.
    ///
    /// This function returns a mutable reference if the type matches.
    pub fn downcast_mut<T: ToStatic>(&mut self) -> Option<&mut RawBlockEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(&mut *(self as *mut BlockEntity<'w, Cx> as *mut RawBlockEntity<'w, T, Cx>))
            }
        } else {
            None
        }
    }

    /// Whether the type of data in this block entity can be safely downcasted
    /// into the target type.
    #[inline]
    pub fn matches_type<T: ToStatic>(&self) -> bool {
        self.data.type_id() == TypeId::of::<T::StaticRepr>()
    }
}

impl<T, Cx> Serialize for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
    T: ?Sized + Serialize,
    Cx::Id: Serialize,
{
    /// Serializes this block entity's data and type id.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &<&RefEntry<_, _>>::from(self.ty))?;
        self.data
            .serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl<'de, T, Cx> rimecraft_serde_update::Update<'de> for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
    T: rimecraft_serde_update::Update<'de> + ?Sized,
{
    #[inline]
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        self.data.update(deserializer)
    }
}

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
{
    type Value = Option<Box<BlockEntity<'w, Cx>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<'a, 'w, Cx>(&'a CreateFromNbt<'w, Cx>)
        where
            Cx: ProvideBlockStateExtTy;

        impl<'de, 'w, Cx> serde::de::Visitor<'de> for Visitor<'_, 'w, Cx>
        where
            Cx: ProvideBlockStateExtTy
                + ProvideRegistry<'w, Cx::Id, RawBlockEntityTypeDyn<'w, Cx>>
                + ProvideNbtTy,
            Cx::Id: Deserialize<'de> + Hash + Eq,
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

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }

                            id = Some(map.next_value()?);
                        }
                        Field::Other(content) => {
                            collect.push(Some((content, map.next_value()?)));
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let registry: &Registry<_, RawBlockEntityTypeDyn<'w, Cx>> = Cx::registry();
                let ty = registry.get(&id).ok_or_else(|| {
                    serde::de::Error::custom(format!("unknown block entity type: {}", id))
                })?;

                let mut be = ty.instantiate(self.0.pos, self.0.state.clone());
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

        deserializer.deserialize_map(Visitor(&self))
    }
}
