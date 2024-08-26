//! `rimecraft-component` integration.

#![cfg(feature = "component")]

use std::sync::LazyLock;

use component::RawErasedComponentType;
use registry::{ProvideRegistry, Registry, RegistryMut};

use crate::{pool::Pool, Id, TestContext};

type Type<'a> = RawErasedComponentType<'a, TestContext>;
type FreezerInstance<'a> = freezer::Freezer<Registry<Id, Type<'a>>, RegistryMut<Id, Type<'a>>>;

static POOL: LazyLock<Pool<FreezerInstance<'static>>> = LazyLock::new(Pool::new);

thread_local! {
    static REGISTRY: &'static FreezerInstance<'static> = unsafe {
        &*POOL.get_or_init(|| crate::integration::registry::registry_freezer("data_component_type"))
    }
}

impl<'a> ProvideRegistry<'a, Id, Type<'static>> for TestContext {
    fn registry() -> &'a Registry<Id, Type<'static>> {
        REGISTRY
            .with(|registry| *registry)
            .get()
            .expect("registry is not initialized")
    }
}

/// Gets the global registry of component types.
///
/// # Panics
///
/// Panics if the registry is not initialized.
pub fn registry() -> &'static Registry<Id, Type<'static>> {
    REGISTRY
        .with(|registry| *registry)
        .get()
        .expect("registry is not initialized")
}

/// Peeks the global mutable registry of component types.
///
/// # Panics
///
/// Panics if the registry is already initialized.
pub fn peek_registry_mut<F>(f: F)
where
    F: FnOnce(&mut RegistryMut<Id, Type<'static>>),
{
    let mut guard = REGISTRY
        .with(|registry| *registry)
        .lock()
        .expect("registry is initialized");
    f(&mut guard)
}

/// Initializes the global registry of component types.
///
/// # Panics
///
/// Panics if the registry is already initialized.
pub fn init_registry() {
    REGISTRY.with(|registry| *registry).freeze(())
}
