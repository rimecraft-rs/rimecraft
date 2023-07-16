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
