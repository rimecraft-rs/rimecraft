use crate::prelude::*;

pub fn root_key() -> Identifier {
    Identifier::parse("root")
}

pub static ITEM: super::Lazy<crate::item::Item> = super::Lazy::new();
pub static BLOCK: super::Lazy<crate::block::Block> = super::Lazy::new();
