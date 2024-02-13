use crate::{Decode, Encode};

#[test]
fn var_i32_ed() {
    use super::VarI32;

    let num = 114514;

    let mut bytes_mut = bytes::BytesMut::new();
    VarI32(num).encode(&mut bytes_mut).unwrap();

    assert_eq!(bytes_mut.len(), VarI32(num).len());

    let mut bytes: bytes::Bytes = bytes_mut.into();
    assert_eq!(VarI32::decode(&mut bytes).unwrap(), num);
}
