use crate::PackedIntArray;

#[test]
fn swap() {
    let mut array = PackedIntArray::from_packed(8, 16, None);
    assert_eq!(array.len(), 16);
    assert_eq!(array.max, u8::MAX as u64);

    array.set(0, 1);
    assert_eq!(array.get(0), Some(1));
    array.set(0, 2);
    assert_eq!(array.get(0), Some(2));
    array.set(15, 255);
    assert_eq!(array.get(15), Some(255));

    assert_eq!(array.set(15, 0), Some(255));
}

#[test]
fn iter() {
    const ARRAY: [u32; 4] = [1, 2, 3, 4];
    let mut array = PackedIntArray::from_packed(8, 4, None);
    for (i, j) in ARRAY.into_iter().enumerate() {
        array.set(i, j);
    }

    let mut iter = array.into_iter();
    assert_eq!(iter.next(), Some(ARRAY[0]));
    assert_eq!(iter.next(), Some(ARRAY[1]));
    assert_eq!(iter.next(), Some(ARRAY[2]));
    assert_eq!(iter.next(), Some(ARRAY[3]));
    assert_eq!(iter.next(), None);
}
