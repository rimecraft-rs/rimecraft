//! Extensions for [`Compound`].

use std::collections::HashMap;

use fastnbt::{ByteArray, IntArray, LongArray, Tag, Value};

/// Represents a nbt compound.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.nbt.NbtCompound` (yarn).
pub type Compound = HashMap<String, Value>;

/// Extension trait for the [`Compound`] type.
///
/// Provides additional methods for inserting and retrieving values from a compound tag.
pub trait CompoundExt {
    /// Inserts an `i8` value into the compound with the specified key.
    fn insert_i8(&mut self, key: String, value: i8);

    /// Inserts an `i16` value into the compound with the specified key.
    fn insert_i16(&mut self, key: String, value: i16);

    /// Inserts an `i32` value into the compound with the specified key.
    fn insert_i32(&mut self, key: String, value: i32);

    /// Inserts an `i64` value into the compound with the specified key.
    fn insert_i64(&mut self, key: String, value: i64);

    /// Inserts an `f32` value into the compound with the specified key.
    fn insert_f32(&mut self, key: String, value: f32);

    /// Inserts an `f64` value into the compound with the specified key.
    fn insert_f64(&mut self, key: String, value: f64);

    /// Inserts a string value into the compound with the specified key.
    fn insert_string(&mut self, key: String, value: String);

    /// Inserts a slice of `i8` values into the compound with the specified key.
    fn insert_i8_slice(&mut self, key: String, value: &[i8]);

    /// Inserts a slice of `i32` values into the compound with the specified key.
    fn insert_i32_slice(&mut self, key: String, value: &[i32]);

    /// Inserts a slice of `i64` values into the compound with the specified key.
    fn insert_i64_slice(&mut self, key: String, value: &[i64]);

    /// Inserts a boolean value into the compound with the specified key.
    /// The boolean value is internally stored as an `i8` (0 for false, 1 for true).
    #[inline]
    fn insert_bool(&mut self, key: String, value: bool) {
        self.insert_i8(key, if value { 1 } else { 0 })
    }

    /// Retrieves the tag with the specified key from the compound.
    fn get_tag(&self, key: &str) -> Option<Tag>;

    /// Checks if the compound contains a tag with the specified key and matching type.
    #[inline]
    fn contains(&self, key: &str, tag: Tag) -> bool {
        self.get_tag(key).map_or(false, |e| e == tag)
    }

    /// Retrieves an `i8` value from the compound with the specified key.
    fn get_i8(&self, key: &str) -> Option<i8>;

    /// Retrieves an `i16` value from the compound with the specified key.
    fn get_i16(&self, key: &str) -> Option<i16>;

    /// Retrieves an `i32` value from the compound with the specified key.
    fn get_i32(&self, key: &str) -> Option<i32>;

    /// Retrieves an `i64` value from the compound with the specified key.
    fn get_i64(&self, key: &str) -> Option<i64>;

    /// Retrieves an `f32` value from the compound with the specified key.
    fn get_f32(&self, key: &str) -> Option<f32>;

    /// Retrieves an `f64` value from the compound with the specified key.
    fn get_f64(&self, key: &str) -> Option<f64>;

    /// Retrieves a string value from the compound with the specified key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Retrieves a slice of `i8` values from the compound with the specified key.
    fn get_i8_slice(&self, key: &str) -> Option<&[i8]>;

    /// Retrieves a slice of `i32` values from the compound with the specified key.
    fn get_i32_slice(&self, key: &str) -> Option<&[i32]>;

    /// Retrieves a slice of `i64` values from the compound with the specified key.
    fn get_i64_slice(&self, key: &str) -> Option<&[i64]>;

    /// Retrieves a nested compound tag from the compound with the specified key.
    fn get_compound(&self, key: &str) -> Option<&Compound>;

    /// Retrieves a slice of `Value` tags from the compound with the specified key.
    fn get_slice(&self, key: &str) -> Option<&[Value]>;

    /// Retrieves a boolean value from the compound with the specified key.
    /// The boolean value is internally stored as an `i8` (0 for false, 1 for true).
    #[inline]
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_i8(key).map(|e| e != 0)
    }
}

impl CompoundExt for Compound {
    #[inline]
    fn insert_i8(&mut self, key: String, value: i8) {
        self.insert(key, Value::Byte(value));
    }

    #[inline]
    fn insert_i16(&mut self, key: String, value: i16) {
        self.insert(key, Value::Short(value));
    }

    #[inline]
    fn insert_i32(&mut self, key: String, value: i32) {
        self.insert(key, Value::Int(value));
    }

    #[inline]
    fn insert_i64(&mut self, key: String, value: i64) {
        self.insert(key, Value::Long(value));
    }

    #[inline]
    fn insert_f32(&mut self, key: String, value: f32) {
        self.insert(key, Value::Float(value));
    }

    #[inline]
    fn insert_f64(&mut self, key: String, value: f64) {
        self.insert(key, Value::Double(value));
    }

    #[inline]
    fn insert_string(&mut self, key: String, value: String) {
        self.insert(key, Value::String(value));
    }

    #[inline]
    fn insert_i8_slice(&mut self, key: String, value: &[i8]) {
        self.insert(key, Value::ByteArray(ByteArray::new(value.into())));
    }

    #[inline]
    fn insert_i32_slice(&mut self, key: String, value: &[i32]) {
        self.insert(key, Value::IntArray(IntArray::new(value.into())));
    }

    #[inline]
    fn insert_i64_slice(&mut self, key: String, value: &[i64]) {
        self.insert(key, Value::LongArray(LongArray::new(value.into())));
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
