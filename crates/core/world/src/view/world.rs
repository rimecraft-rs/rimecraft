//! World views.

use crate::{
    Environment, WorldCx,
    border::{WorldBorder, WorldBorderMut},
    view::{
        HeightLimit,
        block::{BlockEntityView, BlockView, ConstBlockEntityViewMut, ConstBlockViewMut},
        chunk::ChunkView,
        entity::EntityView,
    },
};

/// A view which shows the collision within a world.
pub trait CollisionView<'w, Cx>: BlockView<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// Peeks the world border of this view.
    fn peek_world_border<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&WorldBorder<'w>) -> U;

    /// Peeks the world border mutably of this view.
    fn peek_world_border_mut<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&mut WorldBorderMut<'w>) -> U;
}

/// A scoped view of a world like structure that contains chunks bounded in a dimension.
pub trait WorldView<'w, Cx>:
    ChunkView<'w, Cx>
    + BlockView<'w, Cx>
    + BlockEntityView<'w, Cx>
    + EntityView<'w, Cx>
    + CollisionView<'w, Cx>
where
    Cx: WorldCx<'w>,
{
    /// Returns the top Y level of the given heightmap type at the specified position.
    fn top_y(&self, heightmap_ty: &Cx::HeightmapType, x: i32, y: i32) -> i32;

    /// Returns the runtime environment of this world.
    ///
    /// It is expected that this world is present on a logical server if the value returned [`Environment::Server`].
    fn env(&self) -> Environment;

    /// Returns the height limit of this world.
    fn height_limit(&self) -> HeightLimit;
}

/// [`WorldView`] with mutable access.
pub trait WorldViewMut<'w, Cx>:
    WorldView<'w, Cx> + ConstBlockViewMut<'w, Cx> + ConstBlockEntityViewMut<'w, Cx>
where
    Cx: WorldCx<'w>,
{
}
