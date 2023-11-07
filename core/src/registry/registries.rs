use rimecraft_primitives::{Id, id};

#[inline]
pub fn root_key() -> Id {
    id!("root")
}

pub static ITEM: super::Freezer<crate::item::Item> = super::Freezer::new(super::Builder::new());
pub static BLOCK: super::Freezer<crate::block::Block> = super::Freezer::new(super::Builder::new());
pub static FLUID: super::Freezer<crate::fluid::Fluid> = super::Freezer::new(super::Builder::new());
