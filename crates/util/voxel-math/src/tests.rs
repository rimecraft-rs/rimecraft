use std::ops::ControlFlow;

use glam::{DVec3, dvec3};

use crate::{BBox, BlockPos, raycast};

fn primitive_raycast_collect(src: DVec3, dst: DVec3) -> Box<[BlockPos]> {
    let mut l = vec![];
    raycast(src, dst, |b| {
        l.push(b);
        ControlFlow::<()>::Continue(())
    });
    l.into_boxed_slice()
}

#[test]
fn primitive_raycast_first_hit() {
    assert_eq!(
        *primitive_raycast_collect(dvec3(0.3, 0.5, 0.9), dvec3(0.8, 0.3, 0.1)),
        [BlockPos::new(0, 0, 0)]
    );
}

#[test]
fn primitive_raycast_straight_line_y() {
    assert_eq!(
        *primitive_raycast_collect(dvec3(0.5, 0.5, 0.5), dvec3(0.5, 2.5, 0.5)),
        [
            BlockPos::new(0, 0, 0),
            BlockPos::new(0, 1, 0),
            BlockPos::new(0, 2, 0),
        ]
    );
}

#[test]
fn primitive_raycast_diagonal_xy() {
    let result = primitive_raycast_collect(dvec3(0.1, 0.1, 0.5), dvec3(2.9, 2.9, 0.5));

    // Expected: blocks along a diagonal in X and Y, Z stays 0
    let expected = [
        BlockPos::new(0, 0, 0),
        BlockPos::new(1, 0, 0),
        BlockPos::new(1, 1, 0),
        BlockPos::new(2, 1, 0),
        BlockPos::new(2, 2, 0),
    ];

    assert_eq!(*result, expected);
}

#[test]
fn primitive_raycast_3d_diagonal() {
    let result = primitive_raycast_collect(dvec3(0.1, 0.1, 0.1), dvec3(2.9, 2.9, 2.9));

    // Should visit multiple blocks in all three axes
    assert!(result.len() >= 5);

    // First and last should be correct
    assert_eq!(result[0], BlockPos::new(0, 0, 0));
    assert_eq!(result[result.len() - 1], BlockPos::new(2, 2, 2));

    // Check monotonicity - each coordinate should be non-decreasing
    for i in 1..result.len() {
        assert!(result[i].x() >= result[i - 1].x());
        assert!(result[i].y() >= result[i - 1].y());
        assert!(result[i].z() >= result[i - 1].z());
    }
}

#[test]
fn box_raycast_singular() {
    assert!(
        BBox::new(dvec3(0.1, 0.1, 0.1), dvec3(0.9, 0.9, 0.9))
            .raycast(dvec3(1.5, -0.5, -0.5), dvec3(0.5, 0.5, 0.5))
            .unwrap()
            .distance(dvec3(0.9, 0.1, 0.1))
            <= f64::EPSILON
    );
}

#[test]
fn box_raycast_outside_to_inside() {
    // Ray from left to right through the box
    let src = DVec3::new(-1.0, 0.5, 0.5);
    let dst = DVec3::new(2.0, 0.5, 0.5);

    let result = BBox::new(dvec3(0.0, 0.0, 0.0), dvec3(1.0, 1.0, 1.0)).raycast(src, dst);
    assert!(result.is_some());
    // Should hit at x=0.0
    assert_eq!(result.unwrap().x, 0.0);
}

// inside to outside doesn't work, seems an intended behavior in jcraft
