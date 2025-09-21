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

/// The default max light level of Minecraft.
pub const DEFAULT_MAX_LIGHT_LEVEL: u32 = 15;

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
use rimecraft_voxel_math::direction::Direction;

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
        be_get_game_event_listener => BlockEntityGetGameEventListener<Cx>,
        b_always_replace_state => BlockAlwaysReplaceState,
        b_on_state_replaced => BlockOnStateReplaced<Cx>,
        b_on_block_added => BlockOnBlockAdded<Cx>,
    }
}

/// An utility trait for reborrowing value.
pub trait Reclaim {
    /// The reborrowed value, which can be reborrowed again infinitely.
    type Output<'a>: Reclaim<Output<'a> = Self::Output<'a>>
    where
        Self: 'a;

    /// Reborrows the value.
    fn reclaim(&mut self) -> Self::Output<'_>;
}

impl<'env, T: ?Sized> Reclaim for &'env T {
    type Output<'a>
        = &'env T
    where
        Self: 'a;

    #[inline]
    fn reclaim(&mut self) -> Self::Output<'_> {
        *self
    }
}

impl<T: ?Sized> Reclaim for &mut T {
    type Output<'a>
        = &'a mut T
    where
        Self: 'a;

    #[inline]
    fn reclaim(&mut self) -> Self::Output<'_> {
        *self
    }
}

/// A subset of block state extensions type for use of World OPs.
#[derive(Debug)]
#[non_exhaustive]
pub struct NestedBlockStateExt<'a> {
    culling_shape: Arc<voxel_shape::Slice<'a>>,
    /// Opacity of this block state.
    opacity: u8,
    culling_faces: [Arc<voxel_shape::Slice<'a>>; Direction::COUNT],
    luminance: u32,
}

impl<'a> NestedBlockStateExt<'a> {
    /// Returns the culling face of the given direction.
    #[inline]
    pub fn culling_face(&self, direction: Direction) -> &Arc<voxel_shape::Slice<'a>> {
        &self.culling_faces[direction.ordinal()]
    }

    /// Returns the culling shape of this block state.
    #[inline]
    pub fn culling_shape(&self) -> &Arc<voxel_shape::Slice<'a>> {
        &self.culling_shape
    }

    /// Returns the opacity of this block state.
    #[inline]
    pub fn opacity(&self) -> u8 {
        self.opacity
    }

    /// Returns the luminance of this block state.
    #[inline]
    pub fn luminance(&self) -> u32 {
        self.luminance
    }

    /// Initializes the cached shape and friends of this block state.
    pub fn init_cache(&self) {
        todo!()
    }
}

impl Default for NestedBlockStateExt<'_> {
    fn default() -> Self {
        Self {
            culling_shape: voxel_shape::full_cube().clone(),
            culling_faces: std::array::from_fn(|_| voxel_shape::full_cube().clone()),
            opacity: 15,
            luminance: 0,
        }
    }
}
