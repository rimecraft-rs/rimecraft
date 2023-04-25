use super::{Registry, RegistryKey};
use crate::{item::Item, util::Identifier};
use once_cell::sync::Lazy;

pub static ITEM: Lazy<RegistryKey<Registry<Item>>> = Lazy::new(|| of("item"));

fn of<T>(id: &str) -> RegistryKey<Registry<T>> {
    RegistryKey::of_registry(Identifier::parse(id.to_string()).unwrap())
}
