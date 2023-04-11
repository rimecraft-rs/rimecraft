use crate::util::Rarity;

pub struct Item {
    settings: ItemSettings,
}

impl Item {
    pub fn new(settings: ItemSettings) -> Self {
        Self { settings }
    }

    pub fn get_max_count(&self) -> u32 {
        self.settings.max_count
    }

    pub fn get_max_damage(&self) -> Option<u32> {
        self.settings.max_damage
    }

    pub fn is_damageable(&self) -> bool {
        self.settings.max_damage.is_some()
    }

    pub fn get_recipe_remainder(&self) -> Option<usize> {
        self.settings.recipe_remainder
    }
}

pub struct ItemSettings {
    pub max_count: u32,
    pub max_damage: Option<u32>,
    pub recipe_remainder: Option<usize>,
    pub fireproof: bool,
    pub rarity: Rarity,
}

pub struct FoodComponent {
    pub hunger: i32,
    pub saturation_modifier: f32,
    pub meat: bool,
    pub always_edible: bool,
    pub snack: bool,
    //TODO: statuseffect
}
