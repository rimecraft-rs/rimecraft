#![allow(missing_docs)]
#![cfg(test)]

use registry::{Reg, Registry, RegistryKey as Key, RegistryMut};
use test_global::{Id, TestContext};

#[test]
fn register() {
    let mut registry: RegistryMut<i32, TestContext> = RegistryMut::new(Key::new(
        Id::new("test", "root"),
        Id::new("test", "integer"),
    ));

    assert!(
        registry
            .register(
                Key::new(registry.key().value().clone(), Id::new("test", "one")),
                1
            )
            .is_ok()
    );
    assert!(
        registry
            .register(
                Key::new(registry.key().value().clone(), Id::new("test", "one")),
                1
            )
            .is_err()
    );

    assert!(
        registry
            .register(
                Key::new(registry.key().value().clone(), Id::new("test", "two")),
                2
            )
            .is_ok()
    );
    assert!(
        registry
            .register(
                Key::new(
                    registry.key().value().clone(),
                    Id::new("test", "another_one")
                ),
                1
            )
            .is_ok()
    );
}

#[test]
fn freeze() {
    let mut registry: RegistryMut<i32, TestContext> = RegistryMut::new(Key::new(
        Id::new("test", "root"),
        Id::new("test", "integer"),
    ));

    assert!(
        registry
            .register(
                Key::new(registry.key().value().clone(), Id::new("test", "one")),
                1
            )
            .is_ok()
    );
    assert!(
        registry
            .register(
                Key::new(registry.key().value().clone(), Id::new("test", "two")),
                2
            )
            .is_ok()
    );

    let registry: Registry<_, _> = registry.into();

    assert_eq!(registry.get(&Id::new("test", "one")).unwrap(), 1);
    assert_eq!(registry.get(&Id::new("test", "two")).unwrap(), 2);
    assert!(registry.get(&Id::new("test", "three")).is_none());
}

static_assertions::assert_impl_all!(Registry<i32, TestContext>: Send, Sync, Unpin);
static_assertions::assert_impl_all!(RegistryMut<i32, TestContext>: Send, Sync, Unpin);
static_assertions::assert_impl_all!(Reg<'static, i32, TestContext>: Send, Sync, Unpin);
