use crate::*;

#[test]
fn register() {
    let mut registry: RegistryMut<&'static str, i32> =
        RegistryMut::new(Key::new("root", "integer"));

    assert!(
        registry
            .register(Key::new(registry.key().value(), "one"), 1)
            .is_ok()
    );
    assert!(
        registry
            .register(Key::new(registry.key().value(), "one"), 1)
            .is_err()
    );

    assert!(
        registry
            .register(Key::new(registry.key().value(), "two"), 2)
            .is_ok()
    );
    assert!(
        registry
            .register(Key::new(registry.key().value(), "another_one"), 1)
            .is_ok()
    );
}

#[test]
fn freeze() {
    let mut registry: RegistryMut<&'static str, i32> =
        RegistryMut::new(Key::new("root", "integer"));

    assert!(
        registry
            .register(Key::new(registry.key().value(), "one"), 1)
            .is_ok()
    );
    assert!(
        registry
            .register(Key::new(registry.key().value(), "two"), 2)
            .is_ok()
    );

    let registry: Registry<_, _> = registry.into();

    assert_eq!(registry.get(&"one").unwrap(), 1);
    assert_eq!(registry.get(&"two").unwrap(), 2);
    assert!(registry.get(&"three").is_none());
}

static_assertions::assert_impl_all!(Registry<&'static str, i32>: Send, Sync, Unpin);
static_assertions::assert_impl_all!(RegistryMut<&'static str, i32>: Send, Sync, Unpin);
static_assertions::assert_impl_all!(Reg<'static, &'static str, i32>: Send, Sync, Unpin);
