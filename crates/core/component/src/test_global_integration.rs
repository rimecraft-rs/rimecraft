//! Integration with `rimecraft-test-global`.

#![cfg(feature = "test")]

use test_global::{integration::registry::RegistryType, TestContext};

use crate::RawErasedComponentType;

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct TypeProvider;

impl test_global::integration::registry::GlobalRegistryTypeProvider for TypeProvider {
    type Type<'a> = RawErasedComponentType<'a, TestContext>;
}

test_global::global_registry! {
    type: super::TypeProvider,
    path: "data_component_type",
    mod: _registry
}

impl RegistryType for RawErasedComponentType<'static, TestContext> {
    fn registry() -> &'static rimecraft_registry::Registry<test_global::Id, Self> {
        registry()
    }
}

pub use _registry::*;
