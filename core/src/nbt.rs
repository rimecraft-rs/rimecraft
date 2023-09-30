pub use fastnbt::Tag as NbtType;
pub use fastnbt::Value as NbtElement;

pub use fastnbt::{
    from_bytes, from_bytes_with_opts, from_reader, nbt, to_bytes, to_writer, ByteArray, DeOpts,
    IntArray, LongArray,
};

pub use fastnbt::value::from_value as from_nbt;
pub use fastnbt::value::to_value as to_nbt;

pub use fastsnbt::from_str;

pub type NbtCompound = std::collections::HashMap<String, NbtElement>;

pub type NbtList = Vec<NbtElement>;

/// Extensions for [`NbtCompound`].
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

    #[inline]
    fn insert_bool(&mut self, key: &str, value: bool) {
        self.insert_i8(key, if value { 1 } else { 0 })
    }

    fn get_tag(&self, key: &str) -> Option<NbtType>;

    #[inline]
    fn contains(&self, key: &str, nbt_type: NbtType) -> bool {
        self.get_tag(key).map_or(false, |e| e == nbt_type)
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

    #[inline]
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

    fn get_tag(&self, key: &str) -> Option<NbtType> {
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
        self.get(key).and_then(|e| {
            if let NbtElement::Byte(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    fn get_i16(&self, key: &str) -> Option<i16> {
        self.get(key).and_then(|e| {
            if let NbtElement::Short(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key).and_then(|e| {
            if let NbtElement::Int(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|e| {
            if let NbtElement::Long(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    fn get_f32(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|e| {
            if let NbtElement::Float(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|e| {
            if let NbtElement::Double(value) = e {
                Some(*value)
            } else {
                None
            }
        })
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|e| {
            if let NbtElement::String(value) = e {
                Some(value.as_str())
            } else {
                None
            }
        })
    }

    fn get_i8_slice(&self, key: &str) -> Option<&[i8]> {
        self.get(key).and_then(|e| {
            if let NbtElement::ByteArray(value) = e {
                Some(value.iter().as_slice())
            } else {
                None
            }
        })
    }

    fn get_i32_slice(&self, key: &str) -> Option<&[i32]> {
        self.get(key).and_then(|e| {
            if let NbtElement::IntArray(value) = e {
                Some(value.iter().as_slice())
            } else {
                None
            }
        })
    }

    fn get_i64_slice(&self, key: &str) -> Option<&[i64]> {
        self.get(key).and_then(|e| {
            if let NbtElement::LongArray(value) = e {
                Some(value.iter().as_slice())
            } else {
                None
            }
        })
    }

    fn get_compound(&self, key: &str) -> Option<&NbtCompound> {
        self.get(key).and_then(|e| {
            if let NbtElement::Compound(value) = e {
                Some(value)
            } else {
                None
            }
        })
    }

    fn get_slice(&self, key: &str) -> Option<&[NbtElement]> {
        self.get(key).and_then(|e| {
            if let NbtElement::List(value) = e {
                Some(value.as_slice())
            } else {
                None
            }
        })
    }
}

pub trait Update: serde::Serialize {
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>;
}

impl<T> Update for T
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self = Self::deserialize(deserializer)?;
        Ok(())
    }
}
