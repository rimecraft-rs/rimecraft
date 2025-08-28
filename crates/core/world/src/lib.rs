//! Minecraft world implementation.
//!
//! World generation is not implemented.
//!
//! # The World Lifetime
//!
//! The world lifetime is `'w`, in common. It is the lifetime of the world itself,
//! and `BlockState`s, `FluidState`s and the `Biome` registry should be bound to this lifetime.

pub mod chunk;
pub mod event;
pub mod heightmap;
pub mod tick;
pub mod view;

pub mod behave;

use std::{
    fmt::Debug,
    sync::{Arc, Weak},
};

pub use ahash::{AHashMap, AHashSet};
use parking_lot::Mutex;
use rimecraft_block_entity::BlockEntity;

/// The default max light level of Minecraft.
pub const DEFAULT_MAX_LIGHT_LEVEL: u32 = 15;

/// A sealed cell.
#[derive(Debug)]
#[repr(transparent)]
pub struct Sealed<T>(pub(crate) T);

impl<T> From<T> for Sealed<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self(value)
    }
}

//TODO: PLACEHOLDERS

/// Placeholder of type `ServerWorld`.
pub(crate) type ServerWorld<'w, Cx> = placeholder::ServerWorld<'w, Cx>;

/// Placeholder of type `World`.
pub(crate) type World<'w, Cx> = placeholder::World<'w, Cx>;

/// Placeholder of type `Entity`.
pub(crate) type Entity<'w, Cx> = placeholder::Entity<'w, Cx>; // Should be atomic reference counted with internal mutability.

#[allow(missing_docs, missing_debug_implementations)]
mod placeholder {
    use std::{marker::PhantomData, sync::Arc};

    use crate::chunk::ChunkCx;

    type Invariant<'a> = fn(&'a ()) -> &'a ();

    pub struct ServerWorld<'w, Cx>(PhantomData<(Cx, Invariant<'w>)>);

    impl<'w, Cx> AsRef<World<'w, Cx>> for ServerWorld<'w, Cx>
    where
        Cx: ChunkCx<'w>,
    {
        fn as_ref(&self) -> &World<'w, Cx> {
            &World(PhantomData)
        }
    }

    impl<'w, Cx> ServerWorld<'w, Cx>
    where
        Cx: ChunkCx<'w>,
    {
        pub fn downcast_ref_from_world<'s>(world: &'s World<'w, Cx>) -> Option<&'s Self> {
            let _unused = world;
            unimplemented!("dummy impl")
        }

        pub fn downcast_arc_from_world(world: Arc<World<'w, Cx>>) -> Option<Arc<Self>> {
            let _unused = world;
            unimplemented!("dummy impl")
        }
    }

    pub struct World<'w, Cx>(PhantomData<(Cx, Invariant<'w>)>);

    pub struct Entity<'w, Cx>(PhantomData<&'w Cx>);

    impl<Cx> Clone for Entity<'_, Cx> {
        fn clone(&self) -> Self {
            Self(PhantomData)
        }
    }
}

/// A trait for types that can provide access to an [`Arc`] or [`Weak`].
///
/// This is most useful for self-referential types but need lazy access due to performance considerations.
pub trait ArcAccess<T> {
    /// Returns an [`Arc`] to the wrapped value.
    fn access_arc(self) -> Arc<T>;

    /// Returns a [`Weak`] to the wrapped value.
    fn access_weak(self) -> Weak<T>;
}

impl<T> ArcAccess<T> for Arc<T> {
    #[inline]
    fn access_arc(self) -> Arc<T> {
        self.clone()
    }

    #[inline]
    fn access_weak(self) -> Weak<T> {
        Arc::downgrade(&self)
    }
}

impl<T> ArcAccess<T> for Weak<T> {
    #[inline]
    fn access_arc(self) -> Arc<T> {
        self.upgrade().expect("wrapped value was dropped")
    }

    #[inline]
    fn access_weak(self) -> Weak<T> {
        self.clone()
    }
}

pub(crate) use __dsyn_cache::DsynCache;

mod __dsyn_cache {
    use crate::behave::*;

    use crate::chunk::ChunkCx;

    macro_rules! dsyn_caches_init {
    ($($f:ident=>$t:ty),*$(,)?) => {
        #[derive(Debug)]
        pub(crate) struct DsynCache<'w, Cx>
        where
            Cx: ChunkCx<'w>,
        {
            $($f: std::sync::OnceLock<dsyn::Type<$t>>,)*
            _marker: std::marker::PhantomData<&'w ()>,
        }

        $(
        impl<'w, Cx> local_cx::dsyn::DescriptorTypeCache<$t> for DsynCache<'w, Cx>
        where
            Cx: ChunkCx<'w>,
        {
            #[inline]
            fn get_or_cache<F>(&self, f: F) -> dsyn::Type<$t>
            where
                F: FnOnce() -> dsyn::Type<$t>,
            {
                *self.$f.get_or_init(f)
            }
        }
        )*
    };
    }

    dsyn_caches_init! {
        be_constructor => BlockEntityConstructor<Cx>,
        be_on_block_replaced => BlockEntityOnBlockReplaced<Cx>,
        b_always_replace_state => BlockAlwaysReplaceState,
        b_on_state_replaced => BlockOnStateReplaced<Cx>,
        b_on_block_added => BlockOnBlockAdded<Cx>,
    }
}
