use crate::{BufExt as _, BufMutExt as _, Decode, Encode as _, codecs::ByteArray};

#[test]
fn var_long() {
    let mut buf_mut: Vec<u8> = Vec::new();
    const TEST_VAL: i64 = (u64::MAX - u32::MAX as u64) as i64;
    buf_mut.put_variable(TEST_VAL);
    let mut buf = buf_mut.as_slice();
    assert_eq!(buf.get_variable::<i64>(), TEST_VAL);
}

#[test]
fn byte_array() {
    let bytes = "Hello, World!".as_bytes();
    let mut buf: Vec<u8> = Vec::new();
    ByteArray(bytes).encode(&mut buf).expect("failed to encode");
    let ByteArray(decoded) = ByteArray::<Vec<u8>>::decode(&buf[..]).expect("failed to decode");
    assert_eq!(decoded.as_slice(), bytes);
}

#[test]
fn var_int_signless() {
    let mut buf_mut: Vec<u8> = Vec::new();
    buf_mut.put_variable(114u32);
    let mut buf_alt: Vec<u8> = Vec::new();
    buf_alt.put_variable(114i32);
    assert_eq!(buf_mut, buf_alt);
}

#[test]
fn tuple_ordering() {
    type T = (u8, i32, String);
    let a: T = (13, -10, "3".to_owned());
    let mut buf_mut: Vec<u8> = Vec::new();
    a.encode(&mut buf_mut).expect("encoding err");
    let a_decoded: T = Decode::decode(&*buf_mut).expect("decoding err");
    assert_eq!(a, a_decoded, "decoded tuple does not match original tuple");
}
