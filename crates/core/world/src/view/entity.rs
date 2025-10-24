//! Entity view traits.

use entity::{Entity, EntityCell, EntityCx};
use rimecraft_voxel_math::BBox;

//TODO: use this in practice and check if returning reference is feasible

/// An immutable view of entities contained in a world.
pub trait EntityView<'w, Cx>
where
    Cx: EntityCx<'w>,
{
    /// Returns an iterator over entities filtered by the given `TypeFilter` inside the given bounding box.
    fn entities<'a, Filter>(
        &'a self,
        filter: Filter,
        bbox: BBox,
    ) -> impl Iterator<Item = &'a Filter::Output>
    where
        Filter: entity::TypeFilter<Entity<'w, Cx>>,
        Filter::Output: 'a;

    /// Returns an iterator over players in this view.
    fn players<'a>(&'a self) -> impl Iterator<Item = &'a EntityCell<'w, Cx, Cx::PlayerEntityData>>
    where
        'w: 'a;
}
