use crate::prelude::*;

pub struct Biome {
    weather: Weather,
}

pub struct Weather {
    pub downfall: f32,
    pub has_precipitation: bool,
    pub temp_modifier: TemperatureModifier,
    pub temperature: f32,
}

pub type TemperatureModifier = (&'static str, fn(BlockPos, f32) -> f32);

pub type Shared<'w> = rimecraft_primitives::Ref<'w, crate::registry::Entry<Biome>>;

pub struct SharedRegistry<'w>(pub rimecraft_primitives::Ref<'w, crate::registry::Registry<Biome>>);

impl<'w> rimecraft_collections::Index<Shared<'w>> for SharedRegistry<'w> {
    fn index_of(&self, value: &Shared<'w>) -> Option<usize> {
        self.0
            .iter()
            .position(|entry| rimecraft_primitives::Ref(entry) == *value)
    }

    fn get(&self, index: usize) -> Option<&Shared<'w>> {
        self.0
             .0
            .get_from_raw(index)
            .map(|e| unsafe { &*(e as *const crate::registry::Entry<Biome> as *const Shared<'w>) })
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
