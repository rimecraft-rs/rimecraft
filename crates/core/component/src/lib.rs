//! Minecraft Component implementation.

use std::{any::TypeId, cell::UnsafeCell, fmt::Debug, hash::Hash, marker::PhantomData};

use bytes::{Buf, BufMut};
use edcode2::{Decode, Encode};
use local_cx::{
    ProvideLocalCxTy, WithLocalCx, edcode_codec,
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
pub use local_cx::dyn_codecs::Any;

/// Type of a component data.
///
/// The type `T` should be unique for each component, as it's used to identify the component.
///
/// For the type-erased variant, see [`RawErasedComponentType`].
#[derive(Debug)]
#[doc(alias = "DataComponentType")]
pub struct ComponentType<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    f: Funcs<'a, Cx::LocalContext<'a>>,
    _marker: PhantomData<T>,
}

impl<'a, T, Cx> ComponentType<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    /// Returns whether the component is transient.
    #[inline]
    #[doc(alias = "should_skip_serialization")]
    pub fn is_transient(&self) -> bool {
        self.f.serde_codec.is_none()
    }
    /// Creates a builder of component type.
    #[inline]
    pub const fn builder() -> TypeBuilder<'a, T, Cx> {
        TypeBuilder {
            serde_codec: None,
            packet_codec: None,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, Cx> ComponentType<'a, T, Cx>
where
    T: Clone + Eq + Hash + Debug + Send + Sync + 'a,
    Cx: ProvideLocalCxTy,
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
pub const fn packet_codec_edcode<'a, T, Cx>() -> PacketCodec<'a, T, Cx::LocalContext<'a>>
where
    T: for<'b, 'cx> Encode<WithLocalCx<&'b mut dyn BufMut, Cx::LocalContext<'cx>>>
        + for<'b, 'cx> Decode<'static, WithLocalCx<&'b mut dyn Buf, Cx::LocalContext<'cx>>>
        + Send
        + Sync
        + 'a,
    Cx: ProvideLocalCxTy,
{
    edcode_codec!(T: Any + 'a)
}

/// Creates a new [`PacketCodec`] by NBT serialization.
#[deprecated = "use local-cx-provided macro instead"]
pub const fn packet_codec_nbt<'a, T, Cx>() -> PacketCodec<'a, T, Cx::LocalContext<'a>>
where
    T: Send + Sync + 'a,
    Cx: ProvideLocalCxTy
        + for<'cx> ReadNbtWithCx<T, Cx::LocalContext<'cx>>
        + for<'t, 'cx> WriteNbtWithCx<&'t T, Cx::LocalContext<'cx>>,
{
    edcode_codec!(Nbt<Cx> T: Any + 'a)
}

/// Creates a new [`SerdeCodec`] by using `erased_serde`.
#[deprecated = "use local-cx-provided macro instead"]
pub const fn serde_codec<'a, T, Cx>() -> SerdeCodec<'a, T, Cx::LocalContext<'a>>
where
    for<'t> &'t T: SerializeWithCx<Cx::LocalContext<'t>>,
    T: for<'de, 'cx> DeserializeWithCx<'de, Cx::LocalContext<'cx>> + Send + Sync + 'a,
    Cx: ProvideLocalCxTy,
{
    serde_codec!(T: Any + 'a)
}

/// Builder for creating a new [`ComponentType`].
#[derive(Debug)]
pub struct TypeBuilder<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    serde_codec: Option<UnsafeSerdeCodec<'a, Cx::LocalContext<'a>>>,
    packet_codec: Option<UnsafePacketCodec<'a, Cx::LocalContext<'a>>>,
    _marker: PhantomData<(T, Cx)>,
}

impl<'a, T, Cx> TypeBuilder<'a, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    /// Applies the given serialization and deserialization codec.
    pub const fn serde_codec(self, codec: SerdeCodec<'a, T, Cx::LocalContext<'a>>) -> Self {
        Self {
            serde_codec: Some(codec.codec),
            ..self
        }
    }

    /// Applies the given packet encoding and decoding codec.
    pub const fn packet_codec(self, codec: PacketCodec<'a, T, Cx::LocalContext<'a>>) -> Self {
        Self {
            packet_codec: Some(codec.codec),
            ..self
        }
    }
}

impl<'a, T, Cx> TypeBuilder<'a, T, Cx>
where
    T: Clone + Eq + Hash + Debug + Send + Sync + 'a,
    Cx: ProvideLocalCxTy,
{
    /// Builds a new [`ComponentType`] with the given codecs.
    ///
    /// # Panics
    ///
    /// Panics if the packet codec is not set.
    pub const fn build(self) -> ComponentType<'a, T, Cx> {
        ComponentType {
            f: Funcs {
                serde_codec: self.serde_codec,
                packet_codec: match self.packet_codec {
                    Some(codec) => codec,
                    None => panic!("packet codec is required"),
                },
                util: ComponentType::<T, Cx>::UTIL,
            },
            _marker: PhantomData,
        }
    }
}

impl<T, Cx> Hash for ComponentType<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        typeid::of::<T>().hash(state);
        self.is_transient().hash(state);
    }
}

impl<T, Cx> PartialEq for ComponentType<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.is_transient() == other.is_transient()
    }
}

impl<T, Cx> Copy for ComponentType<'_, T, Cx> where Cx: ProvideLocalCxTy {}

impl<T, Cx> Clone for ComponentType<'_, T, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// [`ComponentType`] with erased type.
///
/// This contains the type ID of the component and the codecs for serialization and packet encoding.
#[derive(Debug)]
pub struct RawErasedComponentType<'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    ty: TypeId,
    f: Funcs<'a, Cx::LocalContext<'a>>,
    _marker: PhantomData<Cx>,
}

/// Codec for serialization and deserialization.
pub type SerdeCodec<'a, T, L> = local_cx::dyn_codecs::SerdeCodec<T, L, Object<'a>, dyn Any + 'a>;
type UnsafeSerdeCodec<'a, L> = local_cx::dyn_codecs::UnsafeSerdeCodec<L, Object<'a>, dyn Any + 'a>;

/// Codec for packet encoding and decoding.
pub type PacketCodec<'a, T, L> = local_cx::dyn_codecs::EdcodeCodec<T, L, Object<'a>, dyn Any + 'a>;
type UnsafePacketCodec<'a, L> =
    local_cx::dyn_codecs::UnsafeEdcodeCodec<L, Object<'a>, dyn Any + 'a>;

#[derive(Debug, Clone, Copy)]
struct DynUtil<'a> {
    clone: fn(&Object<'a>) -> Box<Object<'a>>,
    eq: fn(&'_ Object<'a>, &'_ Object<'a>) -> bool,
    hash: fn(&'_ Object<'a>, &'_ mut dyn std::hash::Hasher),
    dbg: for<'s> fn(&'s Object<'a>) -> &'s (dyn Debug + 'a),
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Funcs<'a, L> {
    serde_codec: Option<UnsafeSerdeCodec<'a, L>>,
    packet_codec: UnsafePacketCodec<'a, L>,
    util: DynUtil<'a>,
}

impl<'a, Cx> RawErasedComponentType<'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    /// Downcasts this type-erased component type into a typed data component type.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'a`.
    #[inline]
    pub unsafe fn downcast<T>(&self) -> Option<ComponentType<'a, T, Cx>> {
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
    pub unsafe fn downcast_unchecked<T>(&self) -> ComponentType<'a, T, Cx> {
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

impl<'a, T, Cx> From<&ComponentType<'a, T, Cx>> for RawErasedComponentType<'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn from(value: &ComponentType<'a, T, Cx>) -> Self {
        Self {
            ty: typeid::of::<T>(),
            f: value.f,
            _marker: PhantomData,
        }
    }
}

impl<Cx> Hash for RawErasedComponentType<'_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ty.hash(state);
        self.is_transient().hash(state);
    }
}

impl<Cx> PartialEq for RawErasedComponentType<'_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty && self.is_transient() == other.is_transient()
    }
}

impl<Cx> Eq for RawErasedComponentType<'_, Cx> where Cx: ProvideLocalCxTy {}

impl<Cx> Copy for RawErasedComponentType<'_, Cx> where Cx: ProvideLocalCxTy {}

impl<Cx> Clone for RawErasedComponentType<'_, Cx>
where
    Cx: ProvideLocalCxTy,
{
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
