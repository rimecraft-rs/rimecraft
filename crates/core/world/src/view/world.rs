//! World views.

use crate::{
    Environment, WorldCx,
    view::{
        HeightLimit,
        block::{BlockEntityView, BlockView},
        chunk::ChunkView,
    },
};

//TODO: do CollisionView
//TODO: do EntityView

/// A scoped view of a world like structure that contains chunks bounded in a dimension.
pub trait WorldView<'w, Cx>:
    ChunkView<'w, Cx> + BlockView<'w, Cx> + BlockEntityView<'w, Cx>
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
