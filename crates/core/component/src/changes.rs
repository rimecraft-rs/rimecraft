//! `ComponentChanges` implementation.

use std::{fmt::Debug, marker::PhantomData, str::FromStr};

use ahash::AHashMap;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_maybe::Maybe;
use rimecraft_registry::{ProvideRegistry, Reg};
use serde::{Deserialize, Serialize};

use crate::{map::CompTyCell, ErasedComponentType, Object, RawErasedComponentType};

/// Changes of components.
pub struct ComponentChanges<'a, 'cow, Cx>
where
    Cx: ProvideIdTy,
{
    pub(crate) changes: Maybe<'cow, AHashMap<CompTyCell<'a, Cx>, Option<Box<Object>>>>,
}

const REMOVED_PREFIX: char = '!';

struct Type<'a, Cx>
where
    Cx: ProvideIdTy,
{
    ty: ErasedComponentType<'a, Cx>,
    rm: bool,
}

impl<Cx> Serialize for Type<'_, Cx>
where
    Cx: ProvideIdTy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let id = Reg::id(self.ty);
        serializer.serialize_str(&if self.rm {
            format!("{}{}", REMOVED_PREFIX, id)
        } else {
            id.to_string()
        })
    }
}

impl<'a, 'de, Cx> Deserialize<'de> for Type<'a, Cx>
where
    Cx: ProvideIdTy + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<Cx>>,
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
            Cx: ProvideIdTy + ProvideRegistry<'a, Cx::Id, RawErasedComponentType<Cx>>,
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

                Ok(Type {
                    ty: Cx::registry().get(&id).ok_or_else(|| {
                        E::custom(format!("unable to find the component type {}", id))
                    })?,
                    rm: stripped.is_some(),
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
        Debug::fmt(&self.changes, f)
    }
}
