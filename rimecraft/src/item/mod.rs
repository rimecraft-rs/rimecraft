use crate::{
    nbt::{compound, NbtCompound, NbtElement},
    registry::registries,
    transfer::{ItemVariant, TransferVariant},
    util::Identifier,
};
use std::cmp::min;

#[derive(Clone)]
pub struct Item {
    max_count: u32,
    max_damage: Option<u32>,
}

impl Item {
    pub fn new(max_count: u32, max_damage: Option<u32>) -> Self {
        Self {
            max_count,
            max_damage: match max_damage {
                None | Some(0) => None,
                Some(c) => Some(c),
            },
        }
    }

    pub fn get_max_count(&self) -> u32 {
        self.max_count
    }

    pub fn get_max_damage(&self) -> Option<u32> {
        self.max_damage
    }

    pub fn is_damageable(&self) -> bool {
        self.max_damage.is_some()
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            max_count: 64,
            max_damage: None,
        }
    }
}

#[derive(Clone)]
pub struct ItemStack {
    variant: ItemVariant,
    count: u32,
    item_cache: Item,
}

impl ItemStack {
    const UNBREAKABLE_KEY: &str = "Unbreakable";
    const DAMAGE_KEY: &str = "Damage";

    pub fn new(item: usize, count: u32, nbt: Option<NbtCompound>) -> Self {
        Self::from_variant(ItemVariant::new(item, nbt), count)
    }

    pub fn from_variant(variant: ItemVariant, count: u32) -> Self {
        let item = registries::ITEM
            .read()
            .unwrap()
            .get_from_raw_id_default(variant.get_raw_id())
            .clone();
        Self {
            variant,
            count,
            item_cache: item,
        }
    }

    pub fn from_nbt(value: &NbtCompound) -> Self {
        let registry = registries::ITEM.read().unwrap();
        let item = registry
            .get_raw_id_from_id(
                &match Identifier::parse(compound::get_str(value, "id").to_string()) {
                    Some(id) => id,
                    None => registry.get_default_id().clone(),
                },
            )
            .unwrap_or(registry.get_default_raw_id());
        let nbt = compound::get_compound(value, "tag").cloned();
        Self::from_variant(
            ItemVariant::new(item, nbt),
            compound::get_u8(value, "Count") as u32,
        )
    }

    pub fn get_variant(&self) -> &ItemVariant {
        &self.variant
    }

    pub fn get_variant_mut(&mut self) -> &mut ItemVariant {
        &mut self.variant
    }

    pub fn get_cached_item(&self) -> &Item {
        &self.item_cache
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.variant.is_blank()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) {
        self.count = count
    }

    pub fn decrement(&mut self, count: u32) {
        self.count -= count
    }

    pub fn split(&mut self, _amount: u32) -> Self {
        let i = min(self.count, self.count);
        let mut stack = self.clone();
        stack.set_count(i);
        self.decrement(i);
        stack
    }

    pub fn clone_and_empty(&mut self) -> ItemStack {
        let stack = self.clone();
        self.set_count(0);
        stack
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let identifier = registries::ITEM
            .read()
            .unwrap()
            .get_entry_from_raw_id(self.variant.get_raw_id())
            .map(|e| e.get_key().unwrap().value.to_string())
            .unwrap_or("rimecraft:air".to_string());
        nbt.insert("id".to_string(), NbtElement::String(identifier));
        nbt.insert("Count".to_string(), NbtElement::U8(self.count as u8));
    }

    pub fn get_max_count(&self) -> u32 {
        self.item_cache.max_count
    }

    pub fn is_stackable(&self) -> bool {
        self.get_max_count() > 1
    }

    pub fn is_damageable(&self) -> bool {
        if self.is_empty()
            || self
                .get_cached_item()
                .get_max_damage()
                .map_or(true, |e| e <= 0)
        {
            false
        } else {
            self.get_variant()
                .get_nbt()
                .map_or(true, |e| !compound::get_bool(e, Self::UNBREAKABLE_KEY))
        }
    }

    pub fn is_damaged(&self) -> bool {
        self.is_damageable() && self.get_damage() > 0
    }

    pub fn get_damage(&self) -> u32 {
        self.variant
            .get_nbt()
            .map_or(0, |nbt| compound::get_i32(nbt, Self::DAMAGE_KEY) as u32)
    }

    pub fn set_damage(&mut self, damage: u32) {
        self.variant
            .get_or_create_nbt_mut()
            .insert(Self::DAMAGE_KEY.to_string(), NbtElement::I32(damage as i32));
    }

    pub fn clone_with_count(&self, count: u32) -> Self {
        let mut stack = self.clone();
        stack.set_count(count);
        stack
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self {
            variant: ItemVariant::default(),
            count: 1,
            item_cache: Item::default(),
        }
    }
}

pub struct FoodComponent {
    pub hunger: i32,
    pub saturation_modifier: f32,
    pub meat: bool,
    pub always_edible: bool,
    pub snack: bool,
    //TODO: statuseffect
}
