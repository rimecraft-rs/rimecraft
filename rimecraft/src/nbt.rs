pub use fastnbt::Tag as NbtType;
pub use fastnbt::Value as NbtElement;

pub use fastnbt::{
    from_bytes, from_bytes_with_opts, from_reader, nbt, to_bytes, to_writer, ByteArray, DeOpts,
    IntArray, LongArray,
};

pub use fastnbt::value::from_value as from_nbt;
pub use fastnbt::value::to_value as to_nbt;

pub use fastsnbt::from_str;

use serde::de::Error;

pub type NbtCompound = std::collections::HashMap<String, NbtElement>;

pub type NbtList = Vec<NbtElement>;

/// Some extensions for [`NbtCompound`].
pub trait NbtCompoundExt {
    fn insert_i8(&mut self, key: &str, value: i8);
    fn insert_i16(&mut self, key: &str, value: i16);
    fn insert_i32(&mut self, key: &str, value: i32);
    fn insert_i64(&mut self, key: &str, value: i64);
    fn insert_f32(&mut self, key: &str, value: f32);
    fn insert_f64(&mut self, key: &str, value: f64);
    fn insert_str(&mut self, key: &str, value: &str);
    fn insert_i8_slice(&mut self, key: &str, value: &[i8]);
    fn insert_i32_slice(&mut self, key: &str, value: &[i32]);
    fn insert_i64_slice(&mut self, key: &str, value: &[i64]);
    fn insert_bool(&mut self, key: &str, value: bool) {
        self.insert_i8(key, if value { 1 } else { 0 })
    }

    fn get_type(&self, key: &str) -> Option<NbtType>;
    fn contains(&self, key: &str, nbt_type: NbtType) -> bool {
        self.get_type(key).map_or(false, |e| e == nbt_type)
    }

    fn get_i8(&self, key: &str) -> Option<i8>;
    fn get_i16(&self, key: &str) -> Option<i16>;
    fn get_i32(&self, key: &str) -> Option<i32>;
    fn get_i64(&self, key: &str) -> Option<i64>;
    fn get_f32(&self, key: &str) -> Option<f32>;
    fn get_f64(&self, key: &str) -> Option<f64>;
    fn get_str(&self, key: &str) -> Option<&str>;
    fn get_i8_slice(&self, key: &str) -> Option<&[i8]>;
    fn get_i32_slice(&self, key: &str) -> Option<&[i32]>;
    fn get_i64_slice(&self, key: &str) -> Option<&[i64]>;
    fn get_compound(&self, key: &str) -> Option<&NbtCompound>;
    fn get_slice(&self, key: &str) -> Option<&[NbtElement]>;
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_i8(key).map(|e| e != 0)
    }
}

impl NbtCompoundExt for NbtCompound {
    fn insert_i8(&mut self, key: &str, value: i8) {
        self.insert(key.to_string(), NbtElement::Byte(value));
    }

    fn insert_i16(&mut self, key: &str, value: i16) {
        self.insert(key.to_string(), NbtElement::Short(value));
    }

    fn insert_i32(&mut self, key: &str, value: i32) {
        self.insert(key.to_string(), NbtElement::Int(value));
    }

    fn insert_i64(&mut self, key: &str, value: i64) {
        self.insert(key.to_string(), NbtElement::Long(value));
    }

    fn insert_f32(&mut self, key: &str, value: f32) {
        self.insert(key.to_string(), NbtElement::Float(value));
    }

    fn insert_f64(&mut self, key: &str, value: f64) {
        self.insert(key.to_string(), NbtElement::Double(value));
    }

    fn insert_str(&mut self, key: &str, value: &str) {
        self.insert(key.to_string(), NbtElement::String(value.to_string()));
    }

    fn insert_i8_slice(&mut self, key: &str, value: &[i8]) {
        self.insert(
            key.to_string(),
            NbtElement::ByteArray(ByteArray::new(Vec::from(value))),
        );
    }

    fn insert_i32_slice(&mut self, key: &str, value: &[i32]) {
        self.insert(
            key.to_string(),
            NbtElement::IntArray(IntArray::new(Vec::from(value))),
        );
    }

    fn insert_i64_slice(&mut self, key: &str, value: &[i64]) {
        self.insert(
            key.to_string(),
            NbtElement::LongArray(LongArray::new(Vec::from(value))),
        );
    }

    fn get_type(&self, key: &str) -> Option<NbtType> {
        self.get(key).map(|e| match e {
            NbtElement::Byte(_) => NbtType::Byte,
            NbtElement::Short(_) => NbtType::Short,
            NbtElement::Int(_) => NbtType::Int,
            NbtElement::Long(_) => NbtType::Long,
            NbtElement::Float(_) => NbtType::Float,
            NbtElement::Double(_) => NbtType::Double,
            NbtElement::String(_) => NbtType::String,
            NbtElement::ByteArray(_) => NbtType::ByteArray,
            NbtElement::IntArray(_) => NbtType::IntArray,
            NbtElement::LongArray(_) => NbtType::LongArray,
            NbtElement::List(_) => NbtType::List,
            NbtElement::Compound(_) => NbtType::Compound,
        })
    }

    fn get_i8(&self, key: &str) -> Option<i8> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Byte(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_i16(&self, key: &str) -> Option<i16> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Short(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Int(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Long(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_f32(&self, key: &str) -> Option<f32> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Float(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Double(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)
            .map(|e| match e {
                NbtElement::String(value) => Some(value.as_str()),
                _ => None,
            })
            .flatten()
    }

    fn get_i8_slice(&self, key: &str) -> Option<&[i8]> {
        self.get(key)
            .map(|e| match e {
                NbtElement::ByteArray(value) => Some(value.iter().as_slice()),
                _ => None,
            })
            .flatten()
    }

    fn get_i32_slice(&self, key: &str) -> Option<&[i32]> {
        self.get(key)
            .map(|e| match e {
                NbtElement::IntArray(value) => Some(value.iter().as_slice()),
                _ => None,
            })
            .flatten()
    }

    fn get_i64_slice(&self, key: &str) -> Option<&[i64]> {
        self.get(key)
            .map(|e| match e {
                NbtElement::LongArray(value) => Some(value.iter().as_slice()),
                _ => None,
            })
            .flatten()
    }

    fn get_compound(&self, key: &str) -> Option<&NbtCompound> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Compound(value) => Some(value),
                _ => None,
            })
            .flatten()
    }

    fn get_slice(&self, key: &str) -> Option<&[NbtElement]> {
        self.get(key)
            .map(|e| match e {
                NbtElement::List(value) => Some(value.as_slice()),
                _ => None,
            })
            .flatten()
    }
}

/// [`fastnbt::input::Input`] implementation for [`bytes::Buf`].
pub struct BufInput<'a, T: bytes::Buf>(pub &'a mut T);

impl<'de, T: bytes::Buf> fastnbt::input::Input<'de> for BufInput<'de, T> {
    fn consume_byte(&mut self) -> fastnbt::error::Result<u8> {
        Ok(self.0.get_u8())
    }

    fn ignore_str(&mut self) -> fastnbt::error::Result<()> {
        let len = self.0.get_u16() as usize;
        self.ignore_bytes(len)
    }

    fn ignore_bytes(&mut self, size: usize) -> fastnbt::error::Result<()> {
        for _ in 0..size {
            self.0.get_u8();
        }
        Ok(())
    }

    fn consume_str<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> fastnbt::error::Result<fastnbt::input::Reference<'de, 's, str>> {
        let n = self.0.get_u16() as usize;
        scratch.clear();
        for i in 0..n {
            scratch[i] = self.0.get_u8();
        }

        let str = cesu8::from_java_cesu8(scratch).map_err(|_| {
            fastnbt::error::Error::custom(format!("Non-unicode string: {:?}", scratch))
        })?;

        Ok(match str {
            std::borrow::Cow::Borrowed(_) => {
                fastnbt::input::Reference::Copied(unsafe { std::str::from_utf8_unchecked(scratch) })
            }
            std::borrow::Cow::Owned(s) => {
                *scratch = s.into_bytes();
                fastnbt::input::Reference::Copied(unsafe { std::str::from_utf8_unchecked(scratch) })
            }
        })
    }

    fn consume_bytes<'s>(
        &'s mut self,
        n: usize,
        scratch: &'s mut Vec<u8>,
    ) -> fastnbt::error::Result<fastnbt::input::Reference<'de, 's, [u8]>> {
        scratch.clear();
        for i in 0..n {
            scratch[i] = self.0.get_u8();
        }
        Ok(fastnbt::input::Reference::Copied(scratch.as_slice()))
    }

    fn consume_i16(&mut self) -> fastnbt::error::Result<i16> {
        Ok(self.0.get_i16())
    }

    fn consume_i32(&mut self) -> fastnbt::error::Result<i32> {
        Ok(self.0.get_i32())
    }

    fn consume_i64(&mut self) -> fastnbt::error::Result<i64> {
        Ok(self.0.get_i64())
    }

    fn consume_f32(&mut self) -> fastnbt::error::Result<f32> {
        Ok(self.0.get_f32())
    }

    fn consume_f64(&mut self) -> fastnbt::error::Result<f64> {
        Ok(self.0.get_f64())
    }
}
