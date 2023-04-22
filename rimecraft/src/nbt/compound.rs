use super::{NbtCompound, NbtElement, NbtList};

pub fn get_u8(nbt: &NbtCompound, key: &str) -> u8 {
    match nbt.get(key) {
        Some(NbtElement::U8(value)) => *value,
        _ => 0,
    }
}

pub fn get_i16(nbt: &NbtCompound, key: &str) -> i16 {
    match nbt.get(key) {
        Some(NbtElement::I16(value)) => *value,
        _ => 0,
    }
}

pub fn get_i32(nbt: &NbtCompound, key: &str) -> i32 {
    match nbt.get(key) {
        Some(NbtElement::I32(value)) => *value,
        _ => 0,
    }
}

pub fn get_i64(nbt: &NbtCompound, key: &str) -> i64 {
    match nbt.get(key) {
        Some(NbtElement::I64(value)) => *value,
        _ => 0,
    }
}

pub fn get_f32(nbt: &NbtCompound, key: &str) -> f32 {
    match nbt.get(key) {
        Some(NbtElement::F32(value)) => *value,
        _ => 0.0,
    }
}

pub fn get_f64(nbt: &NbtCompound, key: &str) -> f64 {
    match nbt.get(key) {
        Some(NbtElement::F64(value)) => *value,
        _ => 0.0,
    }
}

pub fn get_str<'a>(nbt: &'a NbtCompound, key: &str) -> &'a str {
    match nbt.get(key) {
        Some(NbtElement::String(value)) => value,
        _ => "",
    }
}

pub fn get_u8_vec<'a>(nbt: &'a NbtCompound, key: &str) -> Vec<&'a u8> {
    match nbt.get(key) {
        Some(NbtElement::U8Vec(value)) => value.into_iter().collect(),
        _ => Vec::new(),
    }
}

pub fn get_i32_vec<'a>(nbt: &'a NbtCompound, key: &str) -> Vec<&'a i32> {
    match nbt.get(key) {
        Some(NbtElement::I32Vec(value)) => value.into_iter().collect(),
        _ => Vec::new(),
    }
}

pub fn get_i64_vec<'a>(nbt: &'a NbtCompound, key: &str) -> Vec<&'a i64> {
    match nbt.get(key) {
        Some(NbtElement::I64Vec(value)) => value.into_iter().collect(),
        _ => Vec::new(),
    }
}

pub fn get_compound<'a>(nbt: &'a NbtCompound, key: &str) -> Option<&'a NbtCompound> {
    match nbt.get(key) {
        Some(NbtElement::Compound(value)) => Some(value),
        _ => None,
    }
}

pub fn get_list<'a>(nbt: &'a NbtCompound, key: &str) -> Option<&'a NbtList> {
    match nbt.get(key) {
        Some(NbtElement::List(value)) => Some(value),
        _ => None,
    }
}

pub fn get_bool(nbt: &NbtCompound, key: &str) -> bool {
    get_u8(nbt, key) != 0
}
