use crate::{
    item::Item,
    nbt::{NbtCompound, NbtElement},
    network::packet::PacketBytes,
    registry::{registries, DefaultedRegistry, Registry},
};
use bytes::{Buf, BufMut};
use std::ops::Deref;

pub trait TransferVariant<O> {
    fn is_blank(&self) -> bool;
    fn get_raw_id(&self) -> usize;
    fn get_nbt(&self) -> Option<&NbtCompound>;
    fn get_nbt_mut(&mut self) -> Option<&mut NbtCompound>;
    fn to_nbt(&self) -> NbtCompound;
    fn to_packet<T: Buf + BufMut>(&self, buf: &mut PacketBytes<T>);

    fn has_nbt(&self) -> bool {
        self.get_nbt().is_some()
    }

    fn nbt_matches(&self, other: &NbtCompound) -> bool {
        self.get_nbt().map_or_else(|| false, |e| e.eq(other))
    }

    fn clone_nbt(&self) -> Option<NbtCompound> {
        self.get_nbt().map(|b| b.clone())
    }

    fn clone_or_create_nbt(&self) -> NbtCompound {
        self.clone_nbt().unwrap_or(NbtCompound::new())
    }
}

#[derive(PartialEq, Clone)]
pub struct ItemVariant {
    raw_id: usize,
    nbt: Option<NbtCompound>,
}

impl ItemVariant {
    pub fn new(id: usize, nbt: Option<NbtCompound>) -> Self {
        Self { raw_id: id, nbt }
    }

    pub fn set_nbt(&mut self, nbt: Option<NbtCompound>) {
        self.nbt = nbt
    }
}

impl TransferVariant<Item> for ItemVariant {
    fn is_blank(&self) -> bool {
        self.raw_id
            == registries::ITEM
                .lock()
                .unwrap()
                .deref()
                .get_default_raw_id()
    }

    fn get_raw_id(&self) -> usize {
        self.raw_id
    }

    fn get_nbt(&self) -> Option<&NbtCompound> {
        match &self.nbt {
            Some(e) => Some(e),
            None => None,
        }
    }

    fn get_nbt_mut(&mut self) -> Option<&mut NbtCompound> {
        match &mut self.nbt {
            Some(e) => Some(e),
            None => None,
        }
    }

    fn to_nbt(&self) -> NbtCompound {
        let mut result = NbtCompound::new();
        result.insert(
            "item".to_string(),
            NbtElement::String(
                registries::ITEM
                    .lock()
                    .unwrap()
                    .get_entry_from_raw_id(self.raw_id)
                    .unwrap()
                    .get_key()
                    .unwrap()
                    .get_value()
                    .to_string(),
            ),
        );
        if self.nbt.is_some() {
            result.insert(
                "tag".to_string(),
                NbtElement::Compound(self.nbt.clone().unwrap()),
            );
        }
        result
    }

    fn to_packet<T: Buf + BufMut>(&self, buf: &mut PacketBytes<T>) {
        if self.is_blank() {
            buf.put_bool(false);
        } else {
            buf.put_bool(true);
            buf.put_u32(self.raw_id as u32);
            let _ = buf.put_nbt(self.nbt.clone()).unwrap();
        }
    }
}
