//! Minecraft Component implementation.

use std::{any::TypeId, cell::UnsafeCell, fmt::Debug, hash::Hash, marker::PhantomData};

use bytes::{Buf, BufMut};
use edcode2::{Decode, Encode};
use rimecraft_global_cx::{
    nbt::{ReadNbt, UpdateNbt, WriteNbt},
    ProvideIdTy,
};
use rimecraft_registry::{ProvideRegistry, Reg};
use serde::{de::DeserializeOwned, Serialize};

type Object<'a> = dyn Any + Send + Sync + 'a;

pub mod changes;
pub mod map;

mod dyn_any;

pub mod test_global_integration;

use dyn_any::Any;

pub use ahash::{AHashMap, AHashSet};

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

impl<'a, T> ComponentType<'a, T> {
    /// Creates a builder of component type.
    #[inline]
    pub const fn builder<Cx>() -> TypeBuilder<'a, T, Cx> {
        TypeBuilder {
            serde_codec: None,
            packet_codec: None,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> ComponentType<'a, T>
where
    T: Clone + Eq + Hash + Debug + Send + Sync + 'a,
{
    const UTIL: DynUtil<'a> = DynUtil {
        clone: |obj| {
            Box::new(unsafe { &*(std::ptr::from_ref::<Object<'_>>(obj) as *const T) }.clone())
        },
        eq: |a, b| unsafe {
            *(std::ptr::from_ref::<Object<'_>>(a) as *const T)
                == *(std::ptr::from_ref::<Object<'_>>(b) as *const T)
        },
        hash: |obj, mut state| {
            let obj = unsafe { &*(std::ptr::from_ref::<Object<'_>>(obj) as *const T) };
            obj.hash(&mut state);
        },
        dbg: |obj| unsafe {
            &*(std::ptr::from_ref::<Object<'_>>(obj) as *const T as *const (dyn Debug + 'a))
        },
    };
}

/// Creates a new [`PacketCodec`] by encoding and decoding through `edcode2`.
pub const fn packet_codec_edcode<'a, T>() -> PacketCodec<'a, T>
where
    T: for<'b> Encode<&'b mut dyn BufMut>
        + for<'b> Decode<'static, &'b mut dyn Buf>
        + Send
        + Sync
        + 'a,
{
    PacketCodec {
        codec: UnsafePacketCodec {
            encode: |obj, buf| {
                unsafe { &*(std::ptr::from_ref::<Object<'_>>(obj) as *const T) }.encode(buf)
            },
            decode: {
                assert!(
                    <T as Decode<'_, _>>::SUPPORT_NON_IN_PLACE,
                    "non-in-place decoding is not supported for this type",
                );
                |buf| Ok(Box::new(T::decode(buf)?))
            },
            upd: |obj, buf| {
                unsafe { &mut *(std::ptr::from_mut::<Object<'_>>(obj) as *mut T) }
                    .decode_in_place(buf)
            },
        },
        _marker: PhantomData,
    }
}

/// Creates a new [`PacketCodec`] by NBT serialization.
pub const fn packet_codec_nbt<'a, T, Cx>() -> PacketCodec<'a, T>
where
    T: Send + Sync + 'a,
    Cx: ReadNbt<T> + for<'t> WriteNbt<&'t T> + UpdateNbt<T>,
{
    PacketCodec {
        codec: UnsafePacketCodec {
            encode: |obj, buf| {
                Cx::write_nbt(
                    unsafe { &*(std::ptr::from_ref::<Object<'_>>(obj) as *const T) },
                    buf.writer(),
                )
                .map_err(Into::into)
            },
            decode: |buf| Ok(Box::new(Cx::read_nbt(buf.reader())?)),
            upd: |obj, buf| {
                Cx::update_nbt(
                    unsafe { &mut *(std::ptr::from_mut::<Object<'_>>(obj) as *mut T) },
                    buf.reader(),
                )
                .map_err(Into::into)
            },
        },
        _marker: PhantomData,
    }
}

/// Creates a new [`SerdeCodec`] by using `erased_serde`.
pub const fn serde_codec<'a, T>() -> SerdeCodec<'a, T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'a,
{
    SerdeCodec {
        codec: UnsafeSerdeCodec {
            ser: |obj| unsafe {
                &*(std::ptr::from_ref::<Object<'_>>(obj) as *const T
                    as *const (dyn erased_serde::Serialize + 'a))
            },
            de: |deserializer| {
                erased_serde::deserialize::<T>(deserializer).map(|v| {
                    let v: Box<Object<'_>> = Box::new(v);
                    v
                })
            },
            upd: |obj, deserializer| {
                *unsafe { &mut *(std::ptr::from_mut::<Object<'_>>(obj) as *mut T) } =
                    erased_serde::deserialize::<T>(deserializer)?;
                Ok(())
            },
        },
        _marker: PhantomData,
    }
}

/// Builder for creating a new [`ComponentType`].
#[derive(Debug)]
pub struct TypeBuilder<'a, T, Cx> {
    serde_codec: Option<&'a UnsafeSerdeCodec<'a>>,
    packet_codec: Option<&'a UnsafePacketCodec<'a>>,
    _marker: PhantomData<(T, Cx)>,
}

impl<'a, T, Cx> TypeBuilder<'a, T, Cx> {
    /// Applies the given serialization and deserialization codec.
    pub const fn serde_codec(self, codec: &'a SerdeCodec<'a, T>) -> Self {
        Self {
            serde_codec: Some(&codec.codec),
            ..self
        }
    }

    /// Applies the given packet encoding and decoding codec.
    pub const fn packet_codec(self, codec: &'a PacketCodec<'a, T>) -> Self {
        Self {
            packet_codec: Some(&codec.codec),
            ..self
        }
    }
}

impl<'a, T, Cx> TypeBuilder<'a, T, Cx>
where
    T: Clone + Eq + Hash + Debug + Send + Sync + 'a,
{
    /// Builds a new [`ComponentType`] with the given codecs.
    ///
    /// # Panics
    ///
    /// Panics if the packet codec is not set.
    pub const fn build(self) -> ComponentType<'a, T> {
        ComponentType {
            f: Funcs {
                serde_codec: self.serde_codec,
                packet_codec: match self.packet_codec {
                    Some(codec) => codec,
                    None => panic!("packet codec is required"),
                },
                util: &ComponentType::<T>::UTIL,
            },
            _marker: PhantomData,
        }
    }
}

impl<T> Hash for ComponentType<'_, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        typeid::of::<T>().hash(state);
        self.is_transient().hash(state);
    }
}

impl<T> PartialEq for ComponentType<'_, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.is_transient() == other.is_transient()
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

/// Codec for serialization and deserialization.
#[derive(Debug, Clone, Copy)]
pub struct SerdeCodec<'a, T> {
    codec: UnsafeSerdeCodec<'a>,
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct UnsafeSerdeCodec<'a> {
    ser: for<'s> fn(&'s Object<'a>) -> &'s (dyn erased_serde::Serialize + 'a),
    de: fn(&mut dyn erased_serde::Deserializer<'_>) -> erased_serde::Result<Box<Object<'a>>>,
    upd: fn(&mut Object<'a>, &mut dyn erased_serde::Deserializer<'a>) -> erased_serde::Result<()>,
}

/// Codec for packet encoding and decoding.
#[derive(Debug, Clone, Copy)]
pub struct PacketCodec<'a, T> {
    codec: UnsafePacketCodec<'a>,
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct UnsafePacketCodec<'a> {
    encode: fn(&'_ Object<'a>, &'_ mut dyn BufMut) -> Result<(), edcode2::BoxedError<'static>>,
    decode: fn(&'_ mut dyn Buf) -> Result<Box<Object<'a>>, edcode2::BoxedError<'static>>,
    upd: fn(&'_ mut Object<'a>, &'_ mut dyn Buf) -> Result<(), edcode2::BoxedError<'a>>,
}

#[derive(Debug, Clone, Copy)]
struct DynUtil<'a> {
    clone: fn(&Object<'a>) -> Box<Object<'a>>,
    eq: fn(&'_ Object<'a>, &'_ Object<'a>) -> bool,
    hash: fn(&'_ Object<'a>, &'_ mut dyn std::hash::Hasher),
    dbg: for<'s> fn(&'s Object<'a>) -> &'s (dyn Debug + 'a),
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Funcs<'a> {
    serde_codec: Option<&'a UnsafeSerdeCodec<'a>>,
    packet_codec: &'a UnsafePacketCodec<'a>,
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

    /// Returns whether the component is serializable.
    #[inline]
    #[deprecated = "use `is_transient` instead"]
    pub fn is_serializable(&self) -> bool {
        self.f.serde_codec.is_some()
    }

    /// Returns whether the component is transient.
    #[inline]
    pub fn is_transient(&self) -> bool {
        self.f.serde_codec.is_none()
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
        self.is_transient().hash(state);
    }
}

impl<Cx> PartialEq for RawErasedComponentType<'_, Cx> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty && self.is_transient() == other.is_transient()
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

impl<Cx> Copy for RawErasedComponentType<'_, Cx> {}

impl<Cx> Clone for RawErasedComponentType<'_, Cx> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Registration wrapper of [`RawErasedComponentType`].
pub type ErasedComponentType<'a, Cx> =
    Reg<'a, <Cx as ProvideIdTy>::Id, RawErasedComponentType<'a, Cx>>;

struct UnsafeDebugIter<I>(UnsafeCell<I>);

impl<I> Debug for UnsafeDebugIter<I>
where
    I: Iterator<Item: Debug>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let it = &mut *self.0.get();
            f.debug_list().entries(it).finish()
        }
    }
}

#[cfg(all(test, feature = "test"))]
mod tests;
