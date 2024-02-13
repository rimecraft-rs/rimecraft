use super::*;

#[test]
fn swap() {
    let mut packed_array = PackedArray::new(32, 64, None);
    packed_array.iter().for_each(|num| assert_eq!(num, 0));

    assert_eq!(packed_array.swap(4, 1), 0);
    assert_eq!(packed_array.swap(4, 2), 1);

    assert_eq!(packed_array.swap(35, 16), 0);
    assert_eq!(packed_array.swap(35, 7), 16);

    assert_eq!(packed_array.get(4), 2);
    assert_eq!(packed_array.get(35), 7);

    assert!(packed_array.iter().any(|e| e == 2));
    assert!(packed_array.iter().any(|e| e == 7));
}
