//! `rimecraft-registry` integrations.

#![cfg(feature = "registry")]

use crate::Id;

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

/// Generates a registry key for testing.
#[allow(clippy::missing_safety_doc)]
pub const unsafe fn id(value: &'static str) -> Id {
    unsafe {
        Id(identifier::vanilla::Identifier::new(
            identifier::vanilla::Namespace::new_unchecked("test"),
            identifier::vanilla::Path::new_unchecked(value),
        ))
    }
}
