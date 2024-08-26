#![cfg(feature = "registry")]

use freezer::Freezer;
use registry::{Registry, RegistryKey, RegistryMut};

use crate::Id;

impl registry::key::Root for Id {
    fn root() -> Self {
        Self(identifier::vanilla::Identifier::new(
            Default::default(),
            unsafe { identifier::vanilla::Path::new_unchecked("root") },
        ))
    }
}

pub(crate) unsafe fn registry_freezer<T>(
    id: &'static str,
) -> Freezer<Registry<Id, T>, RegistryMut<Id, T>> {
    Freezer::new(RegistryMut::new(RegistryKey::with_root(Id(
        identifier::vanilla::Identifier::new(
            Default::default(),
            identifier::vanilla::Path::new_unchecked(id), // Unsafe OP here
        ),
    ))))
}
