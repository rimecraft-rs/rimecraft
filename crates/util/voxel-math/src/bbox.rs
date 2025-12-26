use std::{fmt::Debug, hash::Hash, ops::Range};

use glam::{DVec2, DVec3, IVec3, Vec3Swizzles as _};
use remap::{remap, remap_method};

use crate::{
    BlockHitResult, BlockPos, F64_TOLERANCE,
    direction::{Axis, AxisDirection, Direction},
};

/// A 3D axis-aligned bounding box.
///
/// The box is defined by its minimum and maximum corners.
#[derive(PartialEq, Clone, Copy)]
#[remap(yarn = "Box", mojmaps = "AABB")]
pub struct BBox {
    min: DVec3,
    max: DVec3,
}

impl BBox {
    /// Creates a new bounding box from two corners.
    #[inline]
    pub fn new<T1, T2>(p1: T1, p2: T2) -> Self
    where
        T1: Into<DVec3>,
        T2: Into<DVec3>,
    {
        let p1 = p1.into();
        let p2 = p2.into();
        Self {
            min: p1.min(p2),
            max: p1.max(p2),
        }
    }

    /// Creates a new bounding box from raw values:
    /// min `x,y,z` and max `x,y,z`.
    #[inline]
    pub fn from_raw(min: DVec3, max: DVec3) -> Self {
        debug_assert!(
            min.cmple(max).all(),
            "min must be less than or equal to max"
        );
        Self { min, max }
    }

    /// Gets the minimum corner of the box.
    #[inline]
    pub fn min(&self) -> DVec3 {
        self.min
    }

    /// Gets the maximum corner of the box.
    #[inline]
    pub fn max(&self) -> DVec3 {
        self.max
    }

    /// Creates a box that is translated by `vec`.
    #[inline]
    #[remap_method(yarn = "offset", mojmaps = "move")]
    pub fn offset(mut self, vec: DVec3) -> Self {
        self.min += vec;
        self.max += vec;
        self
    }

    /// Raycasts through the bounding box using the given ray endpoints, returning the minumum intersection.
    ///
    /// See [`raycast_block`] for raycasting a block in low-level way.
    #[remap_method(yarn = "raycast", mojmaps = "clip")]
    pub fn raycast(&self, src: DVec3, dst: DVec3) -> Option<DVec3> {
        let mut distance = 1.0;
        trace_collision_side(self.min, self.max, src, &mut distance, None, dst - src)
            .is_some()
            .then(|| src + (dst - src) * distance)
    }
}

impl Hash for BBox {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let slice: &[f64; 3] = self.min.as_ref();
        for s in slice {
            s.to_bits().hash(state);
        }
        let slice: &[f64; 3] = self.max.as_ref();
        for s in slice {
            s.to_bits().hash(state);
        }
    }
}

impl Eq for BBox {}

impl From<(DVec3, DVec3)> for BBox {
    #[inline]
    fn from((min, max): (DVec3, DVec3)) -> Self {
        Self::new(min, max)
    }
}

impl From<BlockPos> for BBox {
    #[inline]
    fn from(value: BlockPos) -> Self {
        Self::from_raw(value.0.into(), (value.0 + IVec3::ONE).into())
    }
}

impl From<DVec3> for BBox {
    #[inline]
    fn from(value: DVec3) -> Self {
        Self::new(value, value + DVec3::ONE)
    }
}

impl From<(BlockPos, BlockPos)> for BBox {
    fn from((p1, p2): (BlockPos, BlockPos)) -> Self {
        let v1 = p1.0.min(p2.0);
        let v2 = p1.0.max(p2.0) + IVec3::ONE;
        Self::from_raw(v1.into(), v2.into())
    }
}

impl From<Range<DVec3>> for BBox {
    #[inline]
    fn from(value: Range<DVec3>) -> Self {
        (value.start, value.end).into()
    }
}

impl From<Range<BlockPos>> for BBox {
    #[inline]
    fn from(value: Range<BlockPos>) -> Self {
        (value.start, value.end).into()
    }
}

impl Debug for BBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BBox")
            .field(&self.min.to_array())
            .field(&self.max.to_array())
            .finish()
    }
}

/// Raycasts through an iterator of bounding boxes using the given ray endpoints.
///
/// The boxes should be relative to the given block position, as they are all offseted by `pos` during calculation.
#[remap(yarn = "raycast", mojmaps = "clip")]
pub fn raycast_block<I>(boxes: I, src: DVec3, dst: DVec3, pos: BlockPos) -> Option<BlockHitResult>
where
    I: IntoIterator<Item = BBox>,
{
    let mut distance = 1.0;
    let mut approach = None;
    let delta = dst - src;
    let posd = pos.into();

    for b in boxes {
        let o = b.offset(posd);
        approach = trace_collision_side(o.min(), o.max(), src, &mut distance, approach, delta);
    }

    approach.map(|d| BlockHitResult::new(src + delta * distance, d, pos, false))
}

fn trace_collision_side(
    min: DVec3,
    max: DVec3,
    intersecting: DVec3,
    current_distance: &mut f64,
    mut approach: Option<Direction>,
    delta: DVec3,
) -> Option<Direction> {
    let pos = delta.cmpgt(DVec3::splat(F64_TOLERANCE));
    let trace = pos | delta.cmplt(DVec3::splat(-F64_TOLERANCE));

    if trace.x {
        approach = trace_collision_axis(
            current_distance,
            approach,
            delta,
            if pos.x { min } else { max }.x,
            min.yz(),
            max.yz(),
            Direction::from((AxisDirection::from(pos.x), Axis::X)).opposite(),
            intersecting,
        );
    }
    if trace.y {
        approach = trace_collision_axis(
            current_distance,
            approach,
            delta.yzx(),
            if pos.y { min } else { max }.y,
            min.zx(),
            max.zx(),
            Direction::from((AxisDirection::from(pos.y), Axis::Y)).opposite(),
            intersecting,
        );
    }
    if trace.z {
        approach = trace_collision_axis(
            current_distance,
            approach,
            delta.zxy(),
            if pos.z { min } else { max }.z,
            min.xy(),
            max.xy(),
            Direction::from((AxisDirection::from(pos.z), Axis::Z)).opposite(),
            intersecting,
        );
    }

    approach
}

/// Determines if a ray intersects with a specific side (axis-aligned plane) of a bounding box
/// and updates the closest collision distance.
#[allow(clippy::too_many_arguments)]
fn trace_collision_axis(
    current_distance: &mut f64,
    approach: Option<Direction>,
    direction: DVec3,
    axis_plane: f64,
    min: DVec2,
    max: DVec2,
    normal: Direction,
    origin: DVec3,
) -> Option<Direction> {
    let distance_xplane = (axis_plane - origin.x) / direction.x;
    let intersect_yz = origin.yz() + direction.yz() * distance_xplane;
    if distance_xplane > 0.0
        && *current_distance > distance_xplane
        && (intersect_yz.cmpgt(min - DVec2::splat(F64_TOLERANCE))
            & intersect_yz.cmplt(max + DVec2::splat(F64_TOLERANCE)))
        .all()
    {
        *current_distance = distance_xplane;
        Some(normal)
    } else {
        approach
    }
}
