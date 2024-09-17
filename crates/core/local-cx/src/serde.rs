//! Serde support for local context.

#![cfg(feature = "serde")]
#![allow(clippy::missing_errors_doc)]

use std::marker::PhantomData;

use crate::{BaseLocalContext, WithLocalCx};

/// Serialize the value with a local context.
pub trait SerializeWithCx<LocalCx> {
    /// Serialize the value with the given serializer and the local context.
    fn serialize_with_cx<S>(&self, serializer: WithLocalCx<S, LocalCx>) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}

/// Deserialize the value with a local context.
pub trait DeserializeWithCx<'de, LocalCx>: Sized {
    /// Deserialize the value with the given deserializer and the local context.
    fn deserialize_with_cx<D>(deserializer: WithLocalCx<D, LocalCx>) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;

    /// Deserialize the value in place with the given deserializer and the local context.
    #[inline]
    fn deserialize_in_place_with_cx<D>(
        this: &mut Self,
        deserializer: WithLocalCx<D, LocalCx>,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *this = Self::deserialize_with_cx(deserializer)?;
        Ok(())
    }
}

impl<Cx, T> SerializeWithCx<Cx> for T
where
    T: serde::Serialize + ?Sized,
{
    #[inline]
    fn serialize_with_cx<S>(&self, serializer: WithLocalCx<S, Cx>) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(self, serializer.inner)
    }
}

impl<'de, Cx, T> DeserializeWithCx<'de, Cx> for T
where
    T: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize_with_cx<D>(deserializer: WithLocalCx<D, Cx>) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer.inner)
    }

    #[inline]
    fn deserialize_in_place_with_cx<D>(
        this: &mut Self,
        deserializer: WithLocalCx<D, Cx>,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserialize::deserialize_in_place(deserializer.inner, this)
    }
}

impl<T, Cx> serde::Serialize for WithLocalCx<T, Cx>
where
    T: SerializeWithCx<Cx>,
    Cx: BaseLocalContext,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize_with_cx(WithLocalCx {
            inner: serializer,
            local_cx: self.local_cx,
        })
    }
}

impl<'de, T, Cx> serde::de::DeserializeSeed<'de> for WithLocalCx<PhantomData<T>, Cx>
where
    T: DeserializeWithCx<'de, Cx>,
    Cx: BaseLocalContext,
{
    type Value = T;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize_with_cx(WithLocalCx {
            inner: deserializer,
            local_cx: self.local_cx,
        })
    }
}

#[cfg(feature = "erased-serde")]
mod erased {
    use std::{cell::Cell, marker::PhantomData};

    use crate::{BaseLocalContext, WithLocalCx};

    /// Serialize the value with a local context, with serializer type-erased.
    pub trait ErasedSerializeWithCx<LocalCx> {
        /// Serialize the value with the given serializer and the local context.
        fn erased_serialize_with_cx(
            &self,
            serializer: WithLocalCx<&mut dyn erased_serde::Serializer, &LocalCx>,
        ) -> Result<(), erased_serde::Error>;
    }

    impl<T, Cx> ErasedSerializeWithCx<Cx> for T
    where
        T: super::SerializeWithCx<Cx> + ?Sized,
        Cx: BaseLocalContext,
    {
        fn erased_serialize_with_cx(
            &self,
            serializer: WithLocalCx<&mut dyn erased_serde::Serializer, &Cx>,
        ) -> Result<(), erased_serde::Error> {
            thread_local! {
                static CONTEXT: Cell<*const ()> = const{ Cell::new(std::ptr::null()) };
            }

            struct SerHelper<'a, T: ?Sized, Cx>(&'a T, PhantomData<Cx>);

            impl<T, Cx> serde::Serialize for SerHelper<'_, T, Cx>
            where
                T: super::SerializeWithCx<Cx> + ?Sized,
                Cx: BaseLocalContext,
            {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let ptr: *const Cx = CONTEXT.get().cast();
                    let context = unsafe { *ptr };
                    self.0.serialize_with_cx(WithLocalCx {
                        inner: serializer,
                        local_cx: context,
                    })
                }
            }

            let ptr = std::ptr::from_ref(serializer.local_cx);
            CONTEXT.set(ptr.cast());
            erased_serde::Serialize::erased_serialize(
                &SerHelper(self, PhantomData::<Cx>),
                serializer.inner,
            )
        }
    }
}

#[cfg(feature = "erased-serde")]
pub use erased::ErasedSerializeWithCx;
