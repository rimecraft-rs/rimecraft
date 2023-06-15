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

pub trait NbtCompoundExt {
    fn insert_byte(&mut self, key: &str, value: i8);
    fn insert_short(&mut self, key: &str, value: i16);
    fn insert_int(&mut self, key: &str, value: i32);
    fn insert_long(&mut self, key: &str, value: i64);
    fn insert_float(&mut self, key: &str, value: f32);
    fn insert_double(&mut self, key: &str, value: f64);
    fn insert_string(&mut self, key: &str, value: &str);
    fn insert_byte_array(&mut self, key: &str, value: &[i8]);
    fn insert_int_array(&mut self, key: &str, value: &[i32]);
    fn insert_long_array(&mut self, key: &str, value: &[i64]);
    fn insert_bool(&mut self, key: &str, value: bool) {
        self.insert_byte(key, if value { 1 } else { 0 })
    }

    fn get_type(&self, key: &str) -> Option<NbtType>;
    fn contains(&self, key: &str, nbt_type: NbtType) -> bool {
        self.get_type(key).map_or(false, |e| e == nbt_type)
    }

    fn get_byte(&self, key: &str) -> Option<i8>;
    fn get_short(&self, key: &str) -> Option<i16>;
    fn get_int(&self, key: &str) -> Option<i32>;
    fn get_long(&self, key: &str) -> Option<i64>;
    fn get_float(&self, key: &str) -> Option<f32>;
    fn get_double(&self, key: &str) -> Option<f64>;
    fn get_str(&self, key: &str) -> Option<&str>;
    fn get_byte_array(&self, key: &str) -> Option<&[i8]>;
    fn get_int_array(&self, key: &str) -> Option<&[i32]>;
    fn get_long_array(&self, key: &str) -> Option<&[i64]>;
    fn get_compound(&self, key: &str) -> Option<&NbtCompound>;
    fn get_list(&self, key: &str) -> Option<&[NbtElement]>;
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_byte(key).map(|e| e != 0)
    }
}

impl NbtCompoundExt for NbtCompound {
    fn insert_byte(&mut self, key: &str, value: i8) {
        self.insert(key.to_string(), NbtElement::Byte(value));
    }

    fn insert_short(&mut self, key: &str, value: i16) {
        self.insert(key.to_string(), NbtElement::Short(value));
    }

    fn insert_int(&mut self, key: &str, value: i32) {
        self.insert(key.to_string(), NbtElement::Int(value));
    }

    fn insert_long(&mut self, key: &str, value: i64) {
        self.insert(key.to_string(), NbtElement::Long(value));
    }

    fn insert_float(&mut self, key: &str, value: f32) {
        self.insert(key.to_string(), NbtElement::Float(value));
    }

    fn insert_double(&mut self, key: &str, value: f64) {
        self.insert(key.to_string(), NbtElement::Double(value));
    }

    fn insert_string(&mut self, key: &str, value: &str) {
        self.insert(key.to_string(), NbtElement::String(value.to_string()));
    }

    fn insert_byte_array(&mut self, key: &str, value: &[i8]) {
        self.insert(
            key.to_string(),
            NbtElement::ByteArray(ByteArray::new(Vec::from(value))),
        );
    }

    fn insert_int_array(&mut self, key: &str, value: &[i32]) {
        self.insert(
            key.to_string(),
            NbtElement::IntArray(IntArray::new(Vec::from(value))),
        );
    }

    fn insert_long_array(&mut self, key: &str, value: &[i64]) {
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

    fn get_byte(&self, key: &str) -> Option<i8> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Byte(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_short(&self, key: &str) -> Option<i16> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Short(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_int(&self, key: &str) -> Option<i32> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Int(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_long(&self, key: &str) -> Option<i64> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Long(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_float(&self, key: &str) -> Option<f32> {
        self.get(key)
            .map(|e| match e {
                NbtElement::Float(value) => Some(*value),
                _ => None,
            })
            .flatten()
    }

    fn get_double(&self, key: &str) -> Option<f64> {
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

    fn get_byte_array(&self, key: &str) -> Option<&[i8]> {
        self.get(key)
            .map(|e| match e {
                NbtElement::ByteArray(value) => Some(value.iter().as_slice()),
                _ => None,
            })
            .flatten()
    }

    fn get_int_array(&self, key: &str) -> Option<&[i32]> {
        self.get(key)
            .map(|e| match e {
                NbtElement::IntArray(value) => Some(value.iter().as_slice()),
                _ => None,
            })
            .flatten()
    }

    fn get_long_array(&self, key: &str) -> Option<&[i64]> {
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

    fn get_list(&self, key: &str) -> Option<&[NbtElement]> {
        self.get(key)
            .map(|e| match e {
                NbtElement::List(value) => Some(value.as_slice()),
                _ => None,
            })
            .flatten()
    }
}
