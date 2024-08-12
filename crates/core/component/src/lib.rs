//! Minecraft Component implementation.

use std::{any::TypeId, hash::Hash, marker::PhantomData};

use bytes::{Buf, BufMut};
use edcode2::{Decode, Encode};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{ProvideRegistry, Reg};
use serde::{de::DeserializeOwned, Serialize};

type Object<'a> = dyn Any + Send + Sync + 'a;

pub mod changes;
pub mod map;

mod dyn_any;

use dyn_any::Any;

/// Type of a component data.
///
/// The type `T` should be unique for each component, as it's used to identify the component.
///
/// For the type-erased variant, see [`RawErasedComponentType`].
#[derive(Debug)]
#[doc(alias = "DataComponentType")]
pub struct ComponentType<'a, T> {
    f: Funcs<'a>,
    _marker: PhantomData<T>,
}

impl<T> ComponentType<'_, T> {
    /// Returns whether the component is transient.
    #[inline]
    #[doc(alias = "should_skip_serialization")]
    pub fn is_transient(&self) -> bool {
        self.f.serde_codec.is_none()
    }
}

impl<'a, T> ComponentType<'a, T>
where
    T: Clone + Eq + Send + Sync + 'a,
{
    const UTIL: DynUtil<'a> = DynUtil {
        clone: |obj| Box::new(unsafe { &*(obj as *const Object<'_> as *const T) }.clone()),
        eq: |a, b| unsafe {
            *(a as *const Object<'_> as *const T) == *(b as *const Object<'_> as *const T)
        },
    };
}

impl<'a, T> ComponentType<'a, T>
where
    T: for<'b> Encode<&'b dyn BufMut> + for<'b> Decode<'static, &'b dyn Buf> + Send + Sync + 'a,
{
    /// Codec for packet encoding and decoding.
    pub const PACKET_CODEC: PacketCodec<'a> = PacketCodec {
        encode: |obj, buf| unsafe { &*(obj as *const Object<'_> as *const T) }.encode(buf),
        decode: {
            assert!(
                <T as Decode<'static, &dyn Buf>>::SUPPORT_NON_IN_PLACE,
                "non-in-place decoding is not supported for this type",
            );
            |buf| Ok(Box::new(T::decode(buf)?))
        },
        upd: |obj, buf| unsafe { &mut *(obj as *mut Object<'_> as *mut T) }.decode_in_place(buf),
    };
}

impl<'a, T> ComponentType<'a, T>
where
    T: Clone + Eq + Send + Sync + 'a,
{
    /// Creates a new transient component type.
    ///
    /// Transient components are not serialized.
    #[inline]
    pub const fn transient(packet_codec: Option<&'a PacketCodec<'a>>) -> Self {
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

impl<'a, T> ComponentType<'a, T>
where
    T: Clone + Eq + Serialize + DeserializeOwned + Send + Sync + 'a,
{
    const SERDE_CODEC: SerdeCodec<'a> = SerdeCodec {
        ser: |obj| unsafe { &*(obj as *const Object<'_> as *const T) },
        de: |deserializer| {
            erased_serde::deserialize::<T>(deserializer).map(|v| {
                let v: Box<Object<'_>> = Box::new(v);
                v
            })
        },
        upd: |obj, deserializer| {
            *unsafe { &mut *(obj as *mut Object<'_> as *mut T) } =
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
    pub const fn persistent(packet_codec: Option<&'a PacketCodec<'a>>) -> Self {
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

impl<T> Hash for ComponentType<'_, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        typeid::of::<T>().hash(state);
    }
}

impl<T> PartialEq for ComponentType<'_, T> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<'a, T> Default for ComponentType<'a, T>
where
    T: Clone + Eq + Send + Sync + 'a,
{
    #[inline]
    fn default() -> Self {
        Self::transient(None)
    }
}

impl<T> Copy for ComponentType<'_, T> {}

impl<T> Clone for ComponentType<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// [`ComponentType`] with erased type.
///
/// This contains the type ID of the component and the codecs for serialization and packet encoding.
#[derive(Debug)]
pub struct RawErasedComponentType<'a, Cx> {
    ty: TypeId,
    f: Funcs<'a>,
    _marker: PhantomData<Cx>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct SerdeCodec<'a> {
    ser: for<'s> fn(&'s Object<'a>) -> &'s dyn erased_serde::Serialize,
    de: fn(&mut dyn erased_serde::Deserializer<'_>) -> erased_serde::Result<Box<Object<'a>>>,
    upd: fn(&mut Object<'a>, &mut dyn erased_serde::Deserializer<'a>) -> erased_serde::Result<()>,
}

/// Codec for packet encoding and decoding.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct PacketCodec<'a> {
    encode: fn(&Object<'a>, &mut dyn BufMut) -> Result<(), edcode2::BoxedError<'a>>,
    decode: fn(&mut dyn Buf) -> Result<Box<Object<'a>>, edcode2::BoxedError<'a>>,
    upd: fn(&mut Object<'a>, &mut dyn Buf) -> Result<(), edcode2::BoxedError<'a>>,
}

#[derive(Debug, Clone, Copy)]
struct DynUtil<'a> {
    clone: fn(&Object<'a>) -> Box<Object<'a>>,
    eq: fn(&Object<'a>, &Object<'a>) -> bool,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Funcs<'a> {
    serde_codec: Option<&'a SerdeCodec<'a>>,
    packet_codec: Option<&'a PacketCodec<'a>>,
    util: &'a DynUtil<'a>,
}

impl<'a, Cx> RawErasedComponentType<'a, Cx> {
    /// Downcasts this type-erased component type into a typed data component type.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    #[inline]
    pub unsafe fn downcast<T>(&self) -> Option<ComponentType<'a, T>> {
        (typeid::of::<T>() == self.ty).then_some(ComponentType {
            f: self.f,
            _marker: PhantomData,
        })
    }

    /// Downcasts this type-erased component type into a typed data component type.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    ///
    /// This function does not validate equality of two types.
    #[inline]
    pub unsafe fn downcast_unchecked<T>(&self) -> ComponentType<'a, T> {
        debug_assert_eq!(typeid::of::<T>(), self.ty);
        ComponentType {
            f: self.f,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, Cx> From<&ComponentType<'a, T>> for RawErasedComponentType<'a, Cx> {
    #[inline]
    fn from(value: &ComponentType<'a, T>) -> Self {
        Self {
            ty: typeid::of::<T>(),
            f: value.f,
            _marker: PhantomData,
        }
    }
}

impl<Cx> Hash for RawErasedComponentType<'_, Cx> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ty.hash(state);
    }
}

impl<Cx> PartialEq for RawErasedComponentType<'_, Cx> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

impl<Cx> Eq for RawErasedComponentType<'_, Cx> {}

impl<'r, K, Cx> ProvideRegistry<'r, K, Self> for RawErasedComponentType<'_, Cx>
where
    Cx: ProvideRegistry<'r, K, Self>,
{
    #[inline]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        Cx::registry()
    }
}

/// Registration wrapper of [`RawErasedComponentType`].
pub type ErasedComponentType<'a, Cx> =
    Reg<'a, <Cx as ProvideIdTy>::Id, RawErasedComponentType<'a, Cx>>;
