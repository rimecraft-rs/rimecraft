use crate::collections::Indexed;
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

pub type Shared<'w> = crate::Ref<'w, crate::registry::Entry<Biome>>;

pub struct SharedRegistry<'w>(pub crate::Ref<'w, crate::registry::Registry<Biome>>);

impl<'w> Indexed<Shared<'w>> for SharedRegistry<'w> {
    fn raw_id(&self, value: &Shared<'w>) -> Option<usize> {
        self.0.iter().position(|entry| crate::Ref(entry) == *value)
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
