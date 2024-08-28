//! `rimecraft-registry` integrations.

#![cfg(feature = "registry")]

use registry::ProvideRegistry;

use crate::{Id, TestContext};

impl registry::key::Root for Id {
    fn root() -> Self {
        ROOT_ID
    }
}

/// Root identifier of registry.
pub const ROOT_ID: Id = Id(identifier::vanilla::Identifier::new(
    identifier::vanilla::MINECRAFT,
    unsafe { identifier::vanilla::Path::new_unchecked("root") },
));

/// Provides a registry type by lifetime.
pub trait GlobalRegistryTypeProvider {
    /// Type of the registry type.
    type Type<'a>;
}

/// A valid registry type.
pub trait RegistryType: Sized + 'static {
    /// Gets the registry.
    fn registry() -> &'static registry::Registry<Id, Self>;
}

impl<'a, T> ProvideRegistry<'a, Id, T> for TestContext
where
    T: RegistryType,
{
    fn registry() -> &'a registry::Registry<Id, T> {
        T::registry()
    }
}

/// Generates a global registry module.
#[macro_export]
macro_rules! global_registry {
    (
        type: $p:ty,
        path: $c:literal,
        mod: $m:ident
        $(,)?) => {
        mod $m {
        type Type<'a> = <$p as $crate::integration::registry::GlobalRegistryTypeProvider>::Type<'a>;
        type FreezerInstance<'a> =
            $crate::__priv_freezer::Freezer<
                $crate::__priv_registry::Registry<$crate::Id, Type<'a>>, $crate::__priv_registry::RegistryMut<$crate::Id, Type<'a>>>;

        static __REGISTRY_POOL: ::std::sync::LazyLock<$crate::pool::Pool<FreezerInstance<'static>>> = ::std::sync::LazyLock::new($crate::pool::Pool::new);

        /// Value of the registry key of the registry.
        pub const REGISTRY_ID: $crate::Id = unsafe {
            $crate::Id($crate::__priv_identifier::vanilla::Identifier::new(
                $crate::__priv_identifier::vanilla::Namespace::new_unchecked("test"),
                $crate::__priv_identifier::vanilla::Path::new_unchecked($c),
            ))
        };

        thread_local! {
            static REGISTRY: &'static FreezerInstance<'static> = unsafe {
                &*__REGISTRY_POOL.get_or_init(|| {
                    $crate::__priv_freezer::Freezer::new(
                        $crate::__priv_registry::RegistryMut::new($crate::__priv_registry::RegistryKey::with_root(REGISTRY_ID))
                    )
                })
            }
        }

        /// Gets the global registry.
        ///
        /// # Panics
        ///
        /// Panics if the registry is not initialized.
        pub fn registry() -> &'static $crate::__priv_registry::Registry<$crate::Id, Type<'static>> {
            REGISTRY
                .with(|registry| *registry)
                .get()
                .expect("registry is not initialized")
        }

        /// Peeks the global mutable registry.
        ///
        /// # Panics
        ///
        /// Panics if the registry is already initialized.
        pub fn peek_registry_mut<F>(f: F)
        where
            F: FnOnce(&mut $crate::__priv_registry::RegistryMut<$crate::Id, Type<'static>>),
        {
            let mut guard = REGISTRY
                .with(|registry| *registry)
                .lock()
                .expect("registry is initialized");
            f(&mut guard)
        }

        /// Initializes the global registry.
        ///
        /// # Panics
        ///
        /// Panics if the registry is already initialized.
        pub fn init_registry() {
            REGISTRY.with(|registry| *registry).freeze(())
        }
        }
    };
}
