//! `ComponentChanges` implementation.

use std::{cell::UnsafeCell, fmt::Debug, marker::PhantomData, str::FromStr, sync::OnceLock};

use ahash::AHashMap;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_maybe::Maybe;
use rimecraft_registry::{ProvideRegistry, Reg};
use serde::{Deserialize, Serialize};

use crate::{
    map::CompTyCell, ErasedComponentType, Object, RawErasedComponentType, SerdeCodec,
    UnsafeDebugIter,
};

/// Changes of components.
pub struct ComponentChanges<'a, 'cow, Cx>
where
    Cx: ProvideIdTy,
{
    pub(crate) changed: Maybe<'cow, AHashMap<CompTyCell<'a, Cx>, Option<Box<Object<'a>>>>>,
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
