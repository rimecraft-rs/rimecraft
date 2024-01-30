use std::collections::HashMap;

use fastnbt::{ByteArray, IntArray, LongArray, Tag, Value};

/// Represents a nbt compound.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.nbt.NbtCompound` (yarn).
pub type Compound = HashMap<String, Value>;

/// Extensions for [`Compound`].
pub trait CompoundExt {
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

    #[inline]
    fn insert_bool(&mut self, key: &str, value: bool) {
        self.insert_i8(key, if value { 1 } else { 0 })
    }

    fn get_tag(&self, key: &str) -> Option<Tag>;

    /// Whether the tag matches with
    /// tag stored in this compound.
    #[inline]
    fn contains(&self, key: &str, tag: Tag) -> bool {
        self.get_tag(key).map_or(false, |e| e == tag)
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
    fn get_compound(&self, key: &str) -> Option<&Compound>;
    fn get_slice(&self, key: &str) -> Option<&[Value]>;

    #[inline]
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_i8(key).map(|e| e != 0)
    }
}

impl CompoundExt for Compound {
    #[inline]
    fn insert_i8(&mut self, key: &str, value: i8) {
        self.insert(key.to_string(), Value::Byte(value));
    }

    #[inline]
    fn insert_i16(&mut self, key: &str, value: i16) {
        self.insert(key.to_string(), Value::Short(value));
    }

    #[inline]
    fn insert_i32(&mut self, key: &str, value: i32) {
        self.insert(key.to_string(), Value::Int(value));
    }

    #[inline]
    fn insert_i64(&mut self, key: &str, value: i64) {
        self.insert(key.to_string(), Value::Long(value));
    }

    #[inline]
    fn insert_f32(&mut self, key: &str, value: f32) {
        self.insert(key.to_string(), Value::Float(value));
    }

    #[inline]
    fn insert_f64(&mut self, key: &str, value: f64) {
        self.insert(key.to_string(), Value::Double(value));
    }

    #[inline]
    fn insert_str(&mut self, key: &str, value: &str) {
        self.insert(key.to_string(), Value::String(value.to_string()));
    }

    #[inline]
    fn insert_i8_slice(&mut self, key: &str, value: &[i8]) {
        self.insert(
            key.to_string(),
            Value::ByteArray(ByteArray::new(value.into())),
        );
    }

    #[inline]
    fn insert_i32_slice(&mut self, key: &str, value: &[i32]) {
        self.insert(
            key.to_string(),
            Value::IntArray(IntArray::new(value.into())),
        );
    }

    #[inline]
    fn insert_i64_slice(&mut self, key: &str, value: &[i64]) {
        self.insert(
            key.to_string(),
            Value::LongArray(LongArray::new(value.into())),
        );
    }

    #[inline]
    fn get_tag(&self, key: &str) -> Option<Tag> {
        self.get(key).map(|e| match e {
            Value::Byte(_) => Tag::Byte,
            Value::Short(_) => Tag::Short,
            Value::Int(_) => Tag::Int,
            Value::Long(_) => Tag::Long,
            Value::Float(_) => Tag::Float,
            Value::Double(_) => Tag::Double,
            Value::String(_) => Tag::String,
            Value::ByteArray(_) => Tag::ByteArray,
            Value::IntArray(_) => Tag::IntArray,
            Value::LongArray(_) => Tag::LongArray,
            Value::List(_) => Tag::List,
            Value::Compound(_) => Tag::Compound,
        })
    }

    #[inline]
    fn get_i8(&self, key: &str) -> Option<i8> {
        self.get(key).and_then(|e| {
            if let Value::Byte(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i16(&self, key: &str) -> Option<i16> {
        self.get(key).and_then(|e| {
            if let Value::Short(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key).and_then(|e| {
            if let Value::Int(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|e| {
            if let Value::Long(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_f32(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|e| {
            if let Value::Float(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|e| {
            if let Value::Double(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|e| {
            if let Value::String(value) = e {
                Some(value.as_str())
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i8_slice(&self, key: &str) -> Option<&[i8]> {
        self.get(key).and_then(|e| {
            if let Value::ByteArray(value) = e {
                Some(value.iter().as_slice())
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i32_slice(&self, key: &str) -> Option<&[i32]> {
        self.get(key).and_then(|e| {
            if let Value::IntArray(value) = e {
                Some(value.iter().as_slice())
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i64_slice(&self, key: &str) -> Option<&[i64]> {
        self.get(key).and_then(|e| {
            if let Value::LongArray(value) = e {
                Some(value.iter().as_slice())
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_compound(&self, key: &str) -> Option<&Compound> {
        self.get(key).and_then(|e| {
            if let Value::Compound(value) = e {
                Some(value)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_slice(&self, key: &str) -> Option<&[Value]> {
        self.get(key).and_then(|e| {
            if let Value::List(value) = e {
                Some(value.as_slice())
            } else {
                None
            }
        })
    }
}
