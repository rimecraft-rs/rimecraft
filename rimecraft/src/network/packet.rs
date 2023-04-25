use crate::{
    nbt::{nbt_io, NbtCompound, NbtElement, NbtTagSizeTracker},
    registry::{Registry, RegistryKey},
    util::Identifier,
};
use bytes::{Buf, BufMut};
use datafixerupper::datafixers::util::Either;
use glam::{Quat, Vec3};
use std::{collections::HashMap, hash::Hash, io::Write};
use uuid::Uuid;

pub struct PacketBytes<T: Buf + BufMut> {
    parent: T,
}

impl<T: Buf + BufMut> PacketBytes<T> {
    pub fn get(&self) -> &T {
        &self.parent
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.parent
    }

    pub fn unwrap(self) -> T {
        self.parent
    }

    pub fn new(parent: T) -> Self {
        Self { parent }
    }

    pub fn get_bool(&mut self) -> bool {
        self.get_u8() != 0
    }

    pub fn put_bool(&mut self, value: bool) {
        self.put_u8(if value { 0 } else { 1 })
    }

    pub fn get_vec<C>(&mut self, reader: PacketReader<C, T>) -> Vec<C> {
        let i = self.get_u32();
        let mut vec = Vec::with_capacity(i as usize);
        for _ in 0..i {
            vec.push(reader.apply(self));
        }
        vec
    }

    pub fn put_vec<V>(&mut self, vec: &mut Vec<V>, writer: PacketWriter<V, T>) {
        self.put_u32(vec.len() as u32);
        for _ in 0..vec.len() {
            writer.accept(self, vec.remove(0))
        }
    }

    pub fn get_hash_map<K: PartialEq + Eq + Hash, V>(
        &mut self,
        key_reader: PacketReader<K, T>,
        value_reader: PacketReader<V, T>,
    ) -> HashMap<K, V> {
        let i = self.get_u32();
        let mut map = HashMap::new();
        for _ in 0..i {
            let obj = key_reader.apply(self);
            let obj2 = value_reader.apply(self);
            map.insert(obj, obj2);
        }
        map
    }

    pub fn put_hash_map<K: PartialEq + Eq + Hash, V>(
        &mut self,
        map: HashMap<K, V>,
        key_writer: PacketWriter<K, T>,
        value_writer: PacketWriter<V, T>,
    ) {
        self.put_u32(map.len() as u32);
        for ele in map {
            key_writer.accept(self, ele.0);
            value_writer.accept(self, ele.1);
        }
    }

    pub fn for_each_in_vec(&mut self, consumer: impl Fn(&mut Self)) {
        let i = self.get_u32();
        for _ in 0..i {
            consumer(self)
        }
    }

    pub fn get_option<V>(&mut self, reader: PacketReader<V, T>) -> Option<V> {
        if self.get_bool() {
            Some(reader.apply(self))
        } else {
            None
        }
    }

    pub fn put_option<V>(&mut self, value: Option<V>, writer: PacketWriter<V, T>) {
        if value.is_some() {
            self.put_bool(true);
            writer.accept(self, value.unwrap())
        } else {
            self.put_bool(false)
        }
    }

    pub fn get_either<L, R>(
        &mut self,
        left_reader: PacketReader<L, T>,
        right_reader: PacketReader<R, T>,
    ) -> Either<L, R> {
        if self.get_bool() {
            Either::Left(left_reader.apply(self))
        } else {
            Either::Right(right_reader.apply(self))
        }
    }

    pub fn put_either<L, R>(
        &mut self,
        either: Either<L, R>,
        left_writer: PacketWriter<L, T>,
        right_writer: PacketWriter<R, T>,
    ) {
        match either {
            Either::Left(value) => left_writer.accept(self, value),
            Either::Right(value) => right_writer.accept(self, value),
        }
    }

    pub fn get_vec3(&mut self) -> Vec3 {
        let x = self.get_f32();
        let y = self.get_f32();
        let z = self.get_f32();
        Vec3 { x, y, z }
    }

    pub fn put_vec3(&mut self, vec3: Vec3) {
        self.put_f32(vec3.x);
        self.put_f32(vec3.y);
        self.put_f32(vec3.z);
    }

    pub fn get_quat(&mut self) -> Quat {
        let x = self.get_f32();
        let y = self.get_f32();
        let z = self.get_f32();
        let w = self.get_f32();
        Quat::from_xyzw(x, y, z, w)
    }

    pub fn put_quat(&mut self, quat: Quat) {
        self.put_f32(quat.x);
        self.put_f32(quat.y);
        self.put_f32(quat.z);
        self.put_f32(quat.w);
    }

    pub fn get_uuid(&mut self) -> Uuid {
        let high_bits = self.get_u64();
        let low_bits = self.get_u64();
        Uuid::from_u64_pair(high_bits, low_bits)
    }

    pub fn put_uuid(&mut self, uuid: Uuid) {
        let pair = uuid.as_u64_pair();
        self.put_u64(pair.0);
        self.put_u64(pair.1);
    }

    pub fn get_nbt(
        &mut self,
        size_tracker: &mut NbtTagSizeTracker,
    ) -> crate::Result<Option<NbtCompound>> {
        let b: u8 = self.get_u8();
        if b == 0 {
            return Ok(None);
        }
        nbt_io::read(&mut self.reader(), None, size_tracker)
            .map_err(|o| crate::Error::Encoder(o.to_string()))
            .map(|e| {
                if let NbtElement::Compound(c) = e {
                    Some(c)
                } else {
                    None
                }
            })
    }

    pub fn put_nbt(&mut self, compound: Option<NbtCompound>) -> crate::Result<()> {
        if let Some(e) = compound {
            nbt_io::write(&NbtElement::Compound(e), &mut self.writer())
                .map_err(|o| crate::Error::Encoder(o.to_string()))?;
        } else {
            self.put_u8(0)
        }
        Ok(())
    }

    pub fn get_string(&mut self, max_len: Option<u16>) -> crate::Result<String> {
        let ml = max_len.unwrap_or(32767) as usize;
        let i = ml * 3;
        let j = self.get_u32() as usize;
        if j > i {
            return Err(crate::Error::Decoder(
                format!("The received encoded string buffer length is longer than maximum allowed ({j} > {i})"),
            ));
        }
        let string = {
            let mut vec = Vec::with_capacity(j);
            for _ in 0..j {
                vec.push(self.get_u8())
            }
            String::from_utf8(vec)
        }
        .map_err(|u| crate::Error::Decoder(u.to_string()))?;
        if string.len() > ml {
            return Err(crate::Error::Decoder(format!(
                "The received string length is longer than maximum allowed ({} > {ml})",
                string.len()
            )));
        }
        Ok(string)
    }

    pub fn put_string(&mut self, string: String, max_len: Option<u16>) -> crate::Result<()> {
        let ml = max_len.unwrap_or(32767) as usize;
        let i = ml * 3;
        if string.len() > ml {
            return Err(crate::Error::Encoder(format!(
                "String too big (was {} characters, max {ml})",
                string.len()
            )));
        }
        let bs = string.as_bytes();
        if bs.len() > i {
            return Err(crate::Error::Encoder(format!(
                "String too big (was {} bytes encoded, max {i})",
                bs.len()
            )));
        }
        self.put_u32(bs.len() as u32);
        self.writer()
            .write_all(bs)
            .map_err(|u| crate::Error::Encoder(u.to_string()))?;
        Ok(())
    }

    pub fn get_identifier(&mut self) -> crate::Result<Identifier> {
        match self.get_string(None).map(Identifier::parse) {
            Ok(Some(e)) => Ok(e),
            _ => Err(crate::Error::Decoder("Can't read identifier".to_string())),
        }
    }

    pub fn put_identifier(&mut self, id: Identifier) -> crate::Result<()> {
        self.put_string(id.to_string(), None)
    }

    pub fn get_registry_key<K>(
        &mut self,
        registry_ref: &RegistryKey<Registry<K>>,
    ) -> crate::Result<RegistryKey<K>> {
        let identifier = self.get_identifier()?;
        Ok(RegistryKey::of(registry_ref, identifier))
    }

    pub fn put_registry_key<V>(&mut self, key: RegistryKey<V>) -> crate::Result<()> {
        self.put_identifier(key.value)
    }
}

impl<T: Buf + BufMut> Buf for PacketBytes<T> {
    fn remaining(&self) -> usize {
        self.parent.remaining()
    }

    fn chunk(&self) -> &[u8] {
        self.parent.chunk()
    }

    fn advance(&mut self, cnt: usize) {
        self.parent.advance(cnt)
    }
}

unsafe impl<T: Buf + BufMut> BufMut for PacketBytes<T> {
    fn remaining_mut(&self) -> usize {
        self.parent.remaining_mut()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.parent.advance_mut(cnt)
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        self.parent.chunk_mut()
    }
}

pub struct PacketWriter<T, B: Buf + BufMut>(Box<dyn Fn(&mut PacketBytes<B>, T)>);

impl<T, B: Buf + BufMut> PacketWriter<T, B> {
    pub fn accept(&self, buf: &mut PacketBytes<B>, value: T) {
        self.0(buf, value)
    }
}

pub struct PacketReader<T, B: Buf + BufMut>(Box<dyn Fn(&mut PacketBytes<B>) -> T>);

impl<T, B: Buf + BufMut> PacketReader<T, B> {
    pub fn apply(&self, buf: &mut PacketBytes<B>) -> T {
        self.0(buf)
    }
}
