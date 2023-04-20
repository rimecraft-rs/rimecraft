use crate::{nbt::NbtCompound, transfer::ItemVariant};

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

    pub fn get_variant(&self) -> &ItemVariant {
        &self.variant
    }

    pub fn get_variant_mut(&mut self) -> &mut ItemVariant {
        &mut self.variant
    }
}

impl From<&mut NbtCompound> for ItemStack {
    fn from(value: &mut NbtCompound) -> Self {
        Self { variant: (), count: () }
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
