//! Minecraft Component implementation.

use std::{
    any::{Any, TypeId},
    hash::Hash,
    marker::PhantomData,
};

use rimecraft_edcode::{Decode, Encode};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{ProvideRegistry, Reg};
use serde::{de::DeserializeOwned, Serialize};

type Object = dyn Any + Send + Sync;

pub mod changes;
pub mod map;

/// Type of a component data.
///
/// The type `T` should be unique for each component, as it's used to identify the component.
///
/// For the type-erased variant, see [`RawErasedComponentType`].
#[derive(Debug)]
#[doc(alias = "DataComponentType")]
pub struct ComponentType<T> {
    f: Funcs,
    _marker: PhantomData<T>,
}

impl<T> ComponentType<T> {
    /// Returns whether the component is transient.
    #[inline]
    #[doc(alias = "should_skip_serialization")]
    pub fn is_transient(&self) -> bool {
        self.f.serde_codec.is_none()
    }
}

impl<T> ComponentType<T>
where
    T: Clone + Eq + Send + Sync + 'static,
{
    const UTIL: DynUtil = DynUtil {
        clone: |obj| Box::new(obj.downcast_ref::<T>().expect("mismatched type").clone()),
        eq: |a, b| a.downcast_ref::<T>() == b.downcast_ref::<T>(),
    };
}

impl<T> ComponentType<T>
where
    T: Encode + Decode + Send + Sync + 'static,
{
    /// Codec for packet encoding and decoding.
    pub const PACKET_CODEC: PacketCodec = PacketCodec {
        encode: |obj, buf| {
            obj.downcast_ref::<T>()
                .expect("mismatched type")
                .encode(buf)
        },
        decode: |buf| Ok(Box::new(T::decode(buf)?)),
        upd: |obj, buf| {
            obj.downcast_mut::<T>()
                .expect("mismatched type")
                .decode_in_place(buf)
        },
    };
}

impl<T> ComponentType<T>
where
    T: Clone + Eq + Send + Sync + 'static,
{
    /// Creates a new transient component type.
    ///
    /// Transient components are not serialized.
    ///
    /// This function requires the type to be `'static`. If the type is not `'static`, transmutes
    /// the type to `'static`, which is unsound but works.
    #[inline]
    pub const fn transient(packet_codec: Option<&'static PacketCodec>) -> Self {
        Self {
            f: Funcs {
                serde_codec: None,
                packet_codec,
                util: &Self::UTIL,
            },
            _marker: PhantomData,
        }
    }
}

impl<T> ComponentType<T>
where
    T: Clone + Eq + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    const SERDE_CODEC: SerdeCodec = SerdeCodec {
        ser: |obj| {
            obj.downcast_ref::<T>()
                .expect("the erased type should matches the actual type")
        },
        de: |deserializer| {
            erased_serde::deserialize::<T>(deserializer).map(|v| {
                let v: Box<Object> = Box::new(v);
                v
            })
        },
        upd: |obj, deserializer| {
            *obj.downcast_mut::<T>()
                .expect("the erased type should matches the actual type") =
                erased_serde::deserialize::<T>(deserializer)?;
            Ok(())
        },
    };

    /// Creates a new persistent component type.
    ///
    /// Persistent components are serialized.
    ///
    /// This function requires the type to be `'static`. If the type is not `'static`, transmutes
    /// the type to `'static`, which is unsound but works.
    pub const fn persistent(packet_codec: Option<&'static PacketCodec>) -> Self {
        Self {
            f: Funcs {
                serde_codec: Some(&Self::SERDE_CODEC),
                packet_codec,
                util: &Self::UTIL,
            },
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Hash for ComponentType<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        TypeId::of::<T>().hash(state);
    }
}

impl<T> PartialEq for ComponentType<T> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> ComponentType<T> where T: Encode + Decode {}

impl<T> Default for ComponentType<T>
where
    T: Clone + Eq + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::transient(None)
    }
}

impl<T> Copy for ComponentType<T> {}

impl<T> Clone for ComponentType<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// [`ComponentType`] with erased type.
///
/// This contains the type ID of the component and the codecs for serialization and packet encoding.
#[derive(Debug)]
pub struct RawErasedComponentType<Cx> {
    ty: TypeId,
    f: Funcs,
    _marker: PhantomData<Cx>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct SerdeCodec {
    ser: for<'a> fn(&'a Object) -> &'a dyn erased_serde::Serialize,
    de: fn(&mut dyn erased_serde::Deserializer<'_>) -> erased_serde::Result<Box<Object>>,
    upd: fn(&mut Object, &mut dyn erased_serde::Deserializer<'_>) -> erased_serde::Result<()>,
}

/// Codec for packet encoding and decoding.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct PacketCodec {
    encode: fn(&Object, &mut dyn bytes::BufMut) -> Result<(), std::io::Error>,
    decode: fn(&mut dyn bytes::Buf) -> Result<Box<Object>, std::io::Error>,
    upd: fn(&mut Object, &mut dyn bytes::Buf) -> Result<(), std::io::Error>,
}

#[derive(Debug, Clone, Copy)]
struct DynUtil {
    clone: fn(&Object) -> Box<Object>,
    eq: fn(&Object, &Object) -> bool,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Funcs {
    serde_codec: Option<&'static SerdeCodec>,
    packet_codec: Option<&'static PacketCodec>,
    util: &'static DynUtil,
}

impl<Cx> RawErasedComponentType<Cx> {
    /// Downcasts this type-erased component type into a typed data component type.
    #[inline]
    pub fn downcast<T: 'static>(&self) -> Option<ComponentType<T>> {
        (TypeId::of::<T>() == self.ty).then_some(ComponentType {
            f: self.f,
            _marker: PhantomData,
        })
    }
}

impl<T: 'static, Cx> From<&ComponentType<T>> for RawErasedComponentType<Cx> {
    #[inline]
    fn from(value: &ComponentType<T>) -> Self {
        Self {
            ty: TypeId::of::<T>(),
            f: value.f,
            _marker: PhantomData,
        }
    }
}

impl<Cx> Hash for RawErasedComponentType<Cx> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ty.hash(state);
    }
}

impl<Cx> PartialEq for RawErasedComponentType<Cx> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

impl<Cx> Eq for RawErasedComponentType<Cx> {}

impl<'r, K, Cx> ProvideRegistry<'r, K, Self> for RawErasedComponentType<Cx>
where
    Cx: ProvideRegistry<'r, K, Self>,
{
    #[inline]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        Cx::registry()
    }
}

/// Registration wrapper of [`RawErasedComponentType`].
pub type ErasedComponentType<'a, Cx> = Reg<'a, <Cx as ProvideIdTy>::Id, RawErasedComponentType<Cx>>;
