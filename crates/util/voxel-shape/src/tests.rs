use glam::DVec3;

use crate::{VoxelSet, combine, cuboid, empty, full_cube, set};

#[test]
fn set_boxed() {
    let mut set = VoxelSet::new(set::Props {
        len_x: 16,
        len_y: 16,
        len_z: 16,
    });

    assert!(!set.contains(1, 5, 4));
    set.set(1, 5, 4);
    assert!(set.contains(1, 5, 4));
}

#[test]
fn set_crop() {
    let mut set = VoxelSet::new(set::Props {
        len_x: 16,
        len_y: 16,
        len_z: 16,
    });
    set.set(8, 8, 8);

    let mut cropped = set.crop_mut(set::Bounds {
        x: 4..12,
        y: 4..12,
        z: 4..12,
    });
    assert!(cropped.contains(4, 4, 4));
    cropped.set(1, 3, 5);

    assert!(set.contains(5, 7, 9));
}

#[test]
fn set_voxels() {
    let mut set = VoxelSet::new(set::Props {
        len_x: 16,
        len_y: 16,
        len_z: 16,
    });
    set.set(1, 3, 5);
    set.set(2, 3, 5);
    set.set(1, 4, 5);
    set.set(2, 5, 6);

    let iter = set.voxels();
    assert_eq!(iter.count(), 4, "wrong number of voxels");
}

#[test]
fn set_boxes() {
    let mut set = VoxelSet::new(set::Props {
        len_x: 16,
        len_y: 16,
        len_z: 16,
    });
    set.set(1, 3, 5);
    set.set(2, 3, 5);
    set.set(1, 4, 5);
    set.set(2, 5, 6);

    let iter = set.boxes();
    assert_eq!(iter.count(), 3, "wrong number of boxes");
}

#[test]
fn empty_shape() {
    let empty_shape = empty();
    assert!(empty_shape.is_empty(), "empty shape should be empty")
}

#[test]
fn full_cube_shape() {
    let full_cube = full_cube();
    assert!(
        full_cube.0.__priv_is_cube(),
        "full cube shape should be a cube"
    );
    assert!(!full_cube.is_empty(), "full cube shape should not be empty")
}

#[test]
fn singular_cuboid_boxes() {
    let bbox = (DVec3::new(0.0, 0.0, 0.0), DVec3::new(0.5, 0.25, 0.75)).into();
    let cuboid = cuboid(bbox);
    let mut boxes = cuboid.boxes();
    assert_eq!(
        boxes.next(),
        Some(bbox),
        "singular cuboid boxes should be identical to the original bounding box"
    );
    assert!(
        boxes.next().is_none(),
        "there should be only one element in boxes iter"
    )
}

#[test]
fn combine_bit_accel_cuboid() {
    let bbox0 = (DVec3::new(0.0, 0.0, 0.0), DVec3::new(0.5, 0.25, 0.75)).into();
    let bbox1 = (DVec3::new(0.25, 0.128, 0.875), DVec3::new(1.0, 1.0, 1.0)).into();
    let merged = combine(&cuboid(bbox0), &cuboid(bbox1));
    assert_eq!(
        merged.boxes().count(),
        2,
        "non-mergeable boxes should be an iter with 2 elements"
    );
}

#[test]
fn combine_bit_accel_cuboid_merged() {
    let bbox0 = (DVec3::new(0.0, 0.0, 0.0), DVec3::new(0.5, 0.25, 0.75)).into();
    let bbox1 = (DVec3::new(0.0, 0.0, 0.5), DVec3::new(0.5, 0.25, 0.875)).into();
    let merged = combine(&cuboid(bbox0), &cuboid(bbox1));
    assert_eq!(
        merged.boxes().count(),
        1,
        "mergeable boxes should be an iter with 2 elements"
    );
}

#[test]
fn combine_discrete_cuboid() {
    let bbox0 = (DVec3::new(0.0, 0.0, 0.0), DVec3::new(0.4, 0.35, 0.65)).into();
    let bbox1 = (DVec3::new(0.15, 0.1, 0.85), DVec3::new(1.0, 1.0, 1.0)).into();
    let merged = combine(&cuboid(bbox0), &cuboid(bbox1));
    assert_eq!(
        merged.boxes().count(),
        2,
        "non-mergeable boxes should be an iter with 2 elements"
    );
}

#[test]
fn combine_discrete_cuboid_merged() {
    let bbox0 = (DVec3::new(0.0, 0.0, 0.0), DVec3::new(0.4, 0.35, 0.65)).into();
    let bbox1 = (DVec3::new(0.0, 0.0, 0.55), DVec3::new(0.4, 0.35, 0.875)).into();
    let merged = combine(&cuboid(bbox0), &cuboid(bbox1));
    assert_eq!(
        merged.boxes().count(),
        1,
        "mergeable boxes should be an iter with 2 elements"
    );
}
