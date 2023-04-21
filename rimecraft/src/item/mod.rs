use std::cmp::min;

use crate::{
    nbt::{compound, NbtCompound, NbtElement},
    registry::{registries, DefaultedRegistry, Registry},
    transfer::{ItemVariant, TransferVariant},
    util::Identifier,
};

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
}

impl ItemStack {
    pub fn new(item: usize, count: u32, nbt: Option<NbtCompound>) -> Self {
        Self {
            variant: ItemVariant::new(item, nbt),
            count,
        }
    }

    pub fn from_variant(variant: ItemVariant, count: u32) -> Self {
        Self { variant, count }
    }

    pub fn get_variant(&self) -> &ItemVariant {
        &self.variant
    }

    pub fn get_variant_mut(&mut self) -> &mut ItemVariant {
        &mut self.variant
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

    pub fn split(&mut self, amount: u32) -> Self {
        let i = min(self.count, self.count);
        let mut stack = self.clone();
        stack.set_count(i);
        self.decrement(i);
        stack
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let identifier = registries::ITEM
            .lock()
            .unwrap()
            .get_entry_from_raw_id(self.variant.get_raw_id())
            .map(|e| e.get_key().unwrap().value.to_string())
            .unwrap_or("rimecraft:air".to_string());
        nbt.insert("id".to_string(), NbtElement::String(identifier));
        nbt.insert("Count".to_string(), NbtElement::U8(self.count as u8));
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self {
            variant: ItemVariant::default(),
            count: 1,
        }
    }
}

impl From<&NbtCompound> for ItemStack {
    fn from(value: &NbtCompound) -> Self {
        let registry = registries::ITEM.lock().unwrap();
        let item = registry
            .get_raw_id_from_id(
                match Identifier::parse(compound::get_str(value, "id").to_string()) {
                    Some(id) => &id,
                    None => &registry.get_default_id(),
                },
            )
            .unwrap_or(registry.get_default_raw_id());
        drop(registry);
        let nbt = if value.contains_key("tag") {
            Some(compound::get_compound(value, "tag").clone())
        } else {
            None
        };
        Self {
            variant: ItemVariant::new(item, nbt),
            count: compound::get_u8(value, "Count") as u32,
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
