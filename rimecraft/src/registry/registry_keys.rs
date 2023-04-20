use super::{Registry, RegistryKey, SimpleRegistry};
use crate::{item::Item, util::Identifier};
use once_cell::sync::Lazy;

pub static ITEM: Lazy<RegistryKey<SimpleRegistry<Item>>> = Lazy::new(|| of("item"));

fn of<T, R: Registry<T>>(id: &str) -> RegistryKey<R> {
    RegistryKey::of_registry(Identifier::parse(id.to_string()).unwrap())
}
