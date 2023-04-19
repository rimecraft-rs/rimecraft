use crate::nbt::NbtCompound;

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
    item: usize,
    nbt: NbtCompound,
}

pub struct FoodComponent {
    pub hunger: i32,
    pub saturation_modifier: f32,
    pub meat: bool,
    pub always_edible: bool,
    pub snack: bool,
    //TODO: statuseffect
}
