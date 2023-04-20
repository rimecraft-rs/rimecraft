use super::{registry_keys, SimpleRegistry};
use crate::{item::Item, util::Identifier};
use datafixerupper::serialization::Lifecycle;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub fn root_key() -> Identifier {
    Identifier::parse(String::from("root")).unwrap()
}

pub static ITEM: Lazy<Mutex<SimpleRegistry<Item>>> = Lazy::new(|| {
    Mutex::new(SimpleRegistry::new(
        registry_keys::ITEM.clone(),
        Lifecycle::Stable,
        Some(Identifier::parse("air".to_string()).unwrap()),
    ))
});
