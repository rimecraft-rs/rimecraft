use glam::DVec3;
use rcutil::{Invariant, InvariantLifetime};
use remap::{remap, remap_method};
use voxel_math::HitResult;

use std::fmt::Debug;

use crate::{Entity, EntityCell, EntityCx};

/// Hit result with type of entities.
#[remap(yarn = "EntityHitResult", mojmaps = "EntityHitResult")]
pub struct EntityHitResult<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    entity: EntityCell<'a, Cx>,
    pos: DVec3,
}

impl<'a, Cx> EntityHitResult<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    /// Creates a new entity hit result.
    #[inline]
    pub fn new(entity: EntityCell<'a, Cx>, pos: DVec3) -> Self {
        Self { entity, pos }
    }

    /// Returns the entity held by this hit result.
    #[inline]
    #[remap_method(yarn = "getEntity", mojmaps = "getEntity")]
    pub fn entity(&self) -> &EntityCell<'a, Cx> {
        &self.entity
    }
}

impl<'a, Cx> From<EntityCell<'a, Cx>> for EntityHitResult<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    #[inline]
    fn from(value: EntityCell<'a, Cx>) -> Self {
        let pos = value.lock().pos();
        Self::new(value, pos)
    }
}

impl<'a, Cx> Clone for EntityHitResult<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            entity: self.entity.clone(),
            pos: self.pos,
        }
    }
}

impl<'a, Cx> HitResult for EntityHitResult<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    #[inline]
    fn pos(&self) -> DVec3 {
        self.pos
    }

    #[inline]
    fn is_missed(&self) -> bool {
        false
    }
}

// SAFETY: basically a wrapper around `Entity<'a, Cx>`
unsafe impl<'a, Cx> Invariant for EntityHitResult<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    type Lifetime = InvariantLifetime<'a>;
}

impl<'a, Cx> Debug for EntityHitResult<'a, Cx>
where
    Cx: EntityCx<'a, Id: Debug, Compound: Debug, EntityExt<'a>: Debug>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityHitResult")
            .field("entity", &&self.entity)
            .finish()
    }
}

/// Extensions to [`HitResult`] trait for entity interaction.
pub trait HitResultExt: HitResult {
    /// Returns the squared distance to the target entity.
    #[inline]
    #[remap_method(yarn = "squaredDistanceTo", mojmaps = "distanceTo")]
    fn squared_distance_to<'a, Cx>(&self, entity: &Entity<'a, Cx>) -> f64
    where
        Cx: EntityCx<'a>,
    {
        self.pos().distance_squared(entity.pos())
    }
}

impl<T: ?Sized> HitResultExt for T where T: HitResult {}
