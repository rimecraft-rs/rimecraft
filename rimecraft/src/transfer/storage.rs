use crate::nbt::NbtCompound;

pub trait TransferVariant<O>: Into<NbtCompound> {
    fn is_blank(&self) -> bool;
    fn get_raw_id(&self) -> usize;
    fn get_nbt(&self) -> Option<&NbtCompound>;
    fn get_nbt_mut(&mut self) -> Option<&mut NbtCompound>;

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
