use std::{
    any::TypeId,
    fmt::Debug,
    sync::{Arc, Weak},
};

use local_cx::HoldLocalContext;
use parking_lot::RwLock;
use rimecraft_block_entity::BlockEntityCell;
use rimecraft_registry::RegistryKey;

use crate::{
    ArcAccess, Environment, InvariantLifetime, WorldCx, border::WorldBorderMut, view::HeightLimit,
};

/// Type of light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)] // this could be exhaustive. can't imagine a new light type.
pub enum LightType {
    /// Sky light.
    Sky,
    /// Block luminance.
    Block,
}

/// A base world structure.
pub struct BaseWorld<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// The world border.
    pub border: RwLock<WorldBorderMut<'w>>,

    /// Registry key of this world.
    pub registry_key: RegistryKey<Cx::Id, Self>,
    /// The runtime environment of this world, indicating whether it is a server or a client.
    pub env: Environment,
    /// The local context of this game instance.
    pub local_cx: Cx::LocalContext<'w>,
}

impl<'w, Cx> Debug for BaseWorld<'w, Cx>
where
    Cx: WorldCx<'w>,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseWorld")
            .field("registry_key", &self.registry_key)
            .field("border", &self.border)
            .field("env", &self.env)
            .finish_non_exhaustive()
    }
}

/// A marker trait for world types.
///
/// # Safety
///
/// This trait guarantees the lifetime of world is invariant, and is the lifetime which
/// [`WorldMarker::Lifetime`] is referring to.
pub unsafe trait WorldMarker {
    /// The lifetime of this world.
    ///
    /// This is not present in the generic parameters because it should be unique for each
    /// type with different lifetimes.
    /// In practice this should always be [`InvariantLifetime<'w>`].
    type Lifetime;

    /// Returns the type id of this world type.
    #[inline]
    fn typeid(&self) -> TypeId {
        rcutil::typeid::<Self>()
    }
}

/// An abstracted subset of world behaviors that are essential to chunk operations.
pub trait DynCompatibleWorld<'w, Cx>:
    WorldMarker<Lifetime = InvariantLifetime<'w>> + HoldLocalContext<LocalCx = Cx::LocalContext<'w>>
where
    Cx: WorldCx<'w>,
{
    /// Loads a block entity into this world.
    #[inline]
    fn load_block_entity(&self, be: &BlockEntityCell<'w, Cx>) {
        let _ = (be,);
    }

    /// Returns the environment of this world.
    fn env(&self) -> Environment;

    /// Returns the height limit of this world.
    fn height_limit(&self) -> HeightLimit;
}

unsafe fn downcast_world_const<'w, World>(
    world: *const (dyn WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w),
) -> Option<*const World>
where
    World: WorldMarker<Lifetime = InvariantLifetime<'w>>,
{
    (rcutil::typeid::<World>() == unsafe { &*world }.typeid()).then_some(world.cast())
}

/// Downcasts a world reference to a specific world type.
#[inline]
pub fn downcast_world_ref<'a, 'w, World>(
    world: &'a (dyn WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w),
) -> Option<&'a World>
where
    World: WorldMarker<Lifetime = InvariantLifetime<'w>>,
{
    unsafe { downcast_world_const(world) }.map(|ptr| unsafe { &*ptr })
}

/// Downcasts an arc world reference to a specific world type.
#[inline]
pub fn downcast_world_arc<'w, World>(
    world: Arc<dyn WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w>,
) -> Option<Arc<World>>
where
    World: WorldMarker<Lifetime = InvariantLifetime<'w>>,
{
    unsafe { downcast_world_const(Arc::into_raw(world)) }.map(|ptr| unsafe { Arc::from_raw(ptr) })
}

/// A wrapper of [`ArcAccess`] with downcasing behavior for worlds.
#[derive(Debug)]
pub struct WorldDowncastAccess<Access>(pub Access);

impl<'w, Access, World> ArcAccess<World> for WorldDowncastAccess<Access>
where
    Access: ArcAccess<dyn WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w>,
    World: WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w,
{
    fn access_arc(self) -> Arc<World> {
        downcast_world_arc(self.0.access_arc()).expect("type mismatch")
    }

    fn access_weak(self) -> Weak<World> {
        Arc::downgrade(&self.access_arc())
    }
}

impl<'w, Cx> ArcAccess<dyn WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w>
    for &Weak<dyn DynCompatibleWorld<'w, Cx> + Send + Sync + 'w>
where
    Cx: WorldCx<'w>,
{
    #[inline]
    fn access_arc(self) -> Arc<dyn WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w> {
        ArcAccess::<dyn DynCompatibleWorld<'w, Cx> + Send + Sync + 'w>::access_arc(self)
    }

    #[inline]
    fn access_weak(self) -> Weak<dyn WorldMarker<Lifetime = InvariantLifetime<'w>> + 'w> {
        ArcAccess::<dyn DynCompatibleWorld<'w, Cx> + Send + Sync + 'w>::access_weak(self)
    }
}
