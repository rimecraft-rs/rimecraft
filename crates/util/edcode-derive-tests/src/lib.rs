//! Tests for `rimecraft-edcode-derive` crate.

#[cfg(test)]
mod tests {
    use rimecraft_edcode::{bytes::BytesMut, Decode, Encode};

    #[test]
    #[allow(dead_code)]
    fn derive_enum() {
        #[derive(Encode)]
        #[repr(u8)]
        enum Topics {
            Pearl = 15,
            Lakers = 24,
            Kim = 3,
            Someone = 36,
        }

        let mut buf = BytesMut::new();
        assert!(Topics::Someone.encode(&mut buf).is_ok());
        assert!(<u8 as Decode>::decode(buf).is_ok_and(|x| x == 36));
    }
}
