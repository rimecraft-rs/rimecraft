use super::{registry_keys, Registry};
use crate::{item::Item, util::Identifier};
use datafixerupper::serialization::Lifecycle;
use once_cell::sync::Lazy;
use std::sync::RwLock;

pub fn root_key() -> Identifier {
    Identifier::parse(String::from("root")).unwrap()
}

pub static ITEM: Lazy<RwLock<Registry<Item>>> = Lazy::new(|| {
    RwLock::new(Registry::new(
        registry_keys::ITEM.clone(),
        Lifecycle::Stable,
        Some(Identifier::parse("air".to_string()).unwrap()),
    ))
});
