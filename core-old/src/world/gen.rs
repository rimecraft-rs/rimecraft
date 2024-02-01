use std::fmt::Display;

pub struct StructConfig<'w> {
    biomes: &'w crate::registry::Registry<super::biome::Biome>,
}

pub struct StructSpawns {
    bounding_box: StructSpawnsBoundingBox,
}

pub enum StructSpawnsBoundingBox {
    Piece,
    Struct,
}

impl Display for StructSpawnsBoundingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StructSpawnsBoundingBox::Piece => f.write_str("piece"),
            StructSpawnsBoundingBox::Struct => f.write_str("full"),
        }
    }
}
