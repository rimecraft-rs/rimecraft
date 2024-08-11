use crate::{BufExt, BufMutExt};

#[test]
fn var_long() {
    let mut buf_mut: Vec<u8> = Vec::new();
    const TEST_VAL: i64 = (u64::MAX - u32::MAX as u64) as i64;
    buf_mut.put_variable(TEST_VAL);
    let mut buf = buf_mut.as_slice();
    assert_eq!(buf.get_variable::<i64>(), TEST_VAL);
}
