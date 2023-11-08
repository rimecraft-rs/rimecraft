use formatting::Formatting;

pub mod formatting;
pub mod math;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl From<Rarity> for Formatting {
    #[inline]
    fn from(value: Rarity) -> Self {
        match value {
            Rarity::Common => Formatting::White,
            Rarity::Uncommon => Formatting::Yellow,
            Rarity::Rare => Formatting::Aqua,
            Rarity::Epic => Formatting::LightPurple,
        }
    }
}
