//! Minecraft Component implementation.

use std::{any::TypeId, cell::UnsafeCell, fmt::Debug, hash::Hash, marker::PhantomData};

use bytes::{Buf, BufMut};
use edcode2::{Decode, Encode};
use local_cx::{
    WithLocalCx,
    dyn_codecs::Any,
    dyn_cx::UnsafeDynamicContext,
    edcode_codec,
    nbt::{ReadNbtWithCx, WriteNbtWithCx},
    serde::{DeserializeWithCx, SerializeWithCx},
    serde_codec,
};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

type Object<'a> = dyn Any + Send + Sync + 'a;

pub mod changes;
pub mod map;

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
#[deprecated = "use local-cx-provided macro instead"]
pub const fn packet_codec_edcode<'a, T>() -> PacketCodec<'a, T>
where
    T: for<'b, 'cx> Encode<WithLocalCx<&'b mut dyn BufMut, UnsafeDynamicContext<'cx>>>
        + for<'b, 'cx> Decode<'static, WithLocalCx<&'b mut dyn Buf, UnsafeDynamicContext<'cx>>>
        + Send
        + Sync
        + 'a,
{
    edcode_codec!(T: Any + 'a)
}

/// Creates a new [`PacketCodec`] by NBT serialization.
#[deprecated = "use local-cx-provided macro instead"]
pub const fn packet_codec_nbt<'a, T, Cx>() -> PacketCodec<'a, T>
where
    T: Send + Sync + 'a,
    Cx: for<'cx> ReadNbtWithCx<T, UnsafeDynamicContext<'cx>>
        + for<'t, 'cx> WriteNbtWithCx<&'t T, UnsafeDynamicContext<'cx>>,
{
    edcode_codec!(Nbt<Cx> T: Any + 'a)
}

/// Creates a new [`SerdeCodec`] by using `erased_serde`.
#[deprecated = "use local-cx-provided macro instead"]
pub const fn serde_codec<'a, T>() -> SerdeCodec<'a, T>
where
    for<'t, 'cx> &'t T: SerializeWithCx<UnsafeDynamicContext<'cx>>,
    T: for<'de, 'cx> DeserializeWithCx<'de, UnsafeDynamicContext<'cx>> + Send + Sync + 'a,
{
    serde_codec!(T: Any + 'a)
}

/// Builder for creating a new [`ComponentType`].
#[derive(Debug)]
pub struct TypeBuilder<'a, T, Cx> {
    serde_codec: Option<UnsafeSerdeCodec<'a>>,
    packet_codec: Option<UnsafePacketCodec<'a>>,
    _marker: PhantomData<(T, Cx)>,
}

impl<'a, T, Cx> TypeBuilder<'a, T, Cx> {
    /// Applies the given serialization and deserialization codec.
    pub const fn serde_codec(self, codec: SerdeCodec<'a, T>) -> Self {
        Self {
            serde_codec: Some(codec.codec),
            ..self
        }
    }

    /// Applies the given packet encoding and decoding codec.
    pub const fn packet_codec(self, codec: PacketCodec<'a, T>) -> Self {
        Self {
            packet_codec: Some(codec.codec),
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
                util: ComponentType::<T>::UTIL,
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
pub type SerdeCodec<'a, T> = local_cx::dyn_codecs::SerdeCodec<T, Object<'a>, dyn Any + 'a>;
type UnsafeSerdeCodec<'a> = local_cx::dyn_codecs::UnsafeSerdeCodec<Object<'a>, dyn Any + 'a>;

/// Codec for packet encoding and decoding.
pub type PacketCodec<'a, T> = local_cx::dyn_codecs::EdcodeCodec<T, Object<'a>, dyn Any + 'a>;
type UnsafePacketCodec<'a> = local_cx::dyn_codecs::UnsafeEdcodeCodec<Object<'a>, dyn Any + 'a>;

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
    serde_codec: Option<UnsafeSerdeCodec<'a>>,
    packet_codec: UnsafePacketCodec<'a>,
    util: DynUtil<'a>,
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

#[cfg(test)]
mod tests;
