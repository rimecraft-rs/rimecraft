use crate::{VoxelSet, empty, full_cube, set};

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
