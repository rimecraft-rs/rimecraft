//! Tests for `rimecraft-edcode2-derive` crate.

#![allow(deprecated)]

#[cfg(test)]
mod tests {
    use rimecraft_edcode2::{Decode, Encode};

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

        let mut buf: Vec<u8> = Vec::new();
        assert!(Topics::Someone.encode(&mut buf).is_ok());
        assert!(Topics::decode(buf.as_ref()).is_ok_and(|x| x == Topics::Someone));
    }
}
