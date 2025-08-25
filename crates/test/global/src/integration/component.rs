//! `rimecraft-component` integrations.

#![cfg(feature = "component")]

use component::RawErasedComponentType;
use local_cx::LocalContext;
use registry::{Registry, RegistryKey, RegistryMut};

use crate::{Id, LocalTestContext, TestContext, integration};

/// Default components registry builder.
pub fn default_components_registry_builder<'a>()
-> RegistryMut<Id, RawErasedComponentType<'a, TestContext>> {
    RegistryMut::new(RegistryKey::new(
        integration::registry::ROOT_ID,
        Id::new("test", "components"),
    ))
}

impl<'a> LocalContext<&'a Registry<Id, RawErasedComponentType<'a, TestContext>>>
    for LocalTestContext<'a>
{
    #[inline]
    fn acquire(self) -> &'a Registry<Id, RawErasedComponentType<'a, TestContext>> {
        &self.reg_components
    }
}
