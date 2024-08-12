//! Tests for `rimecraft-edcode-derive` crate.

#![allow(deprecated)]

#[cfg(test)]
mod tests {
    use rimecraft_edcode::{bytes::BytesMut, Decode, Encode};

    #[test]
    #[allow(dead_code)]
    fn derive_enum() {
        #[derive(Encode, Decode, PartialEq, Eq)]
        #[repr(u8)]
        enum Topics {
            Pearl = 15,
            Lakers = 24,
            Kim = 3,
            Someone = 36,
        }

        let mut buf = BytesMut::new();
        assert!(Topics::Someone.encode(&mut buf).is_ok());
        assert!(Topics::decode(buf).is_ok_and(|x| x == Topics::Someone));
    }
}
