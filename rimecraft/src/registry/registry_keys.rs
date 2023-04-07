use once_cell::sync::Lazy;

use crate::util::Identifier;

use super::{Registry, RegistryKey, SimpleRegistry};

pub static ITEM: Lazy<RegistryKey<SimpleRegistry<String>>> = Lazy::new(|| of("item"));

fn of<T, R>(id: &str) -> RegistryKey<R>
where
    R: Registry<T>,
{
    RegistryKey::of_registry(Identifier::parse(id.to_string()).unwrap())
}
