//! `ComponentChanges` implementation.

use std::{cell::UnsafeCell, fmt::Debug, marker::PhantomData, str::FromStr, sync::OnceLock};

use ahash::AHashMap;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_maybe::{Maybe, SimpleOwned};
use rimecraft_registry::{ProvideRegistry, Reg};
use serde::{Deserialize, Serialize};

use crate::{
    map::CompTyCell, ErasedComponentType, Object, RawErasedComponentType, UnsafeDebugIter,
};

/// Changes of components.
pub struct ComponentChanges<'a, 'cow, Cx>
where
    Cx: ProvideIdTy,
{
    pub(crate) changed: Maybe<'cow, AHashMap<CompTyCell<'a, Cx>, Option<Box<Object<'a>>>>>,
}

impl<'a, Cx> ComponentChanges<'a, '_, Cx>
where
    Cx: ProvideIdTy,
{
    /// Returns a builder for `ComponentChanges`.
    pub fn builder() -> Builder<'a, Cx> {
        Builder {
            changes: AHashMap::new(),
        }
    }
}

/// Builder for [`ComponentChanges`].
pub struct Builder<'a, Cx>
where
    Cx: ProvideIdTy,
{
    changes: AHashMap<CompTyCell<'a, Cx>, Option<Box<Object<'a>>>>,
}

impl<'a, Cx> Builder<'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Inserts a component type with a valid value.
    ///
    /// # Panics
    ///
    /// Panics if the type of the value does not match the component type.
    #[inline]
    pub fn insert<T>(&mut self, ty: ErasedComponentType<'a, Cx>, value: T)
    where
        T: Send + Sync + 'a,
    {
        assert_eq!(
            ty.ty,
            typeid::of::<T>(),
            "the type {} does not match the component type",
            std::any::type_name::<T>()
        );
        self.changes.insert(CompTyCell(ty), Some(Box::new(value)));
    }

    /// Inserts a component type with an empty value.
    #[inline]
    pub fn remove(&mut self, ty: ErasedComponentType<'a, Cx>) {
        self.changes.insert(CompTyCell(ty), None);
    }

    /// Builds the changes into a [`ComponentChanges`].
    #[inline]
    pub fn build<'cow>(self) -> ComponentChanges<'a, 'cow, Cx> {
        ComponentChanges {
            changed: Maybe::Owned(SimpleOwned(self.changes)),
        }
    }
}

impl<'a, Cx> From<Builder<'a, Cx>> for ComponentChanges<'a, '_, Cx>
where
    Cx: ProvideIdTy,
{
    #[inline]
    fn from(builder: Builder<'a, Cx>) -> Self {
        builder.build()
    }
}

const REMOVED_PREFIX: char = '!';

struct Type<'a, Cx>
where
    Cx: ProvideIdTy,
{
    ty: ErasedComponentType<'a, Cx>,
    rm: bool,

    cached_ser: OnceLock<String>,
}

impl<Cx> Serialize for Type<'_, Cx>
where
    Cx: ProvideIdTy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.cached_ser.get_or_init(|| {
            let id = Reg::id(self.ty);
            if self.rm {
                format!("{}{}", REMOVED_PREFIX, id)
            } else {
                id.to_string()
            }
        }))
    }
}

impl<'a, 'de, Cx> Deserialize<'de> for Type<'a, Cx>
where
    Cx: ProvideIdTy + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<'a, Cx>>,
    Cx::Id: FromStr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a, Cx>
        where
            Cx: ProvideIdTy,
        {
            _marker: PhantomData<&'a Cx>,
        }

        impl<'a, Cx> serde::de::Visitor<'_> for Visitor<'a, Cx>
        where
            Cx: ProvideIdTy + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<'a, Cx>>,
            Cx::Id: FromStr,
        {
            type Value = Type<'a, Cx>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a string")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let stripped = value.strip_prefix(REMOVED_PREFIX);
                let any = stripped.unwrap_or(value);
                let id: Cx::Id = any.parse().ok().ok_or_else(|| {
                    E::custom(format!("unable to deserialize the identifier {}", any))
                })?;

                let ty = Cx::registry().get(&id).ok_or_else(|| {
                    E::custom(format!("unable to find the component type {}", id))
                })?;

                if !ty.is_serializable() {
                    return Err(E::custom(format!(
                        "the component type {} is not serializable",
                        id
                    )));
                }

                Ok(Type {
                    ty,
                    rm: stripped.is_some(),

                    cached_ser: OnceLock::new(),
                })
            }
        }

        deserializer.deserialize_str(Visitor {
            _marker: PhantomData,
        })
    }
}

impl<Cx> Debug for ComponentChanges<'_, '_, Cx>
where
    Cx: ProvideIdTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&UnsafeDebugIter(UnsafeCell::new(self.changed.keys())), f)
    }
}

impl<Cx> Debug for Builder<'_, Cx>
where
    Cx: ProvideIdTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&UnsafeDebugIter(UnsafeCell::new(self.changes.keys())), f)
    }
}
