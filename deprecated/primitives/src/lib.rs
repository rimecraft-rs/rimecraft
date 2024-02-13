pub mod identifier;
pub mod reference;

#[cfg(feature = "serde")]
pub use rimecraft_serde_update as serde_update;

#[cfg(test)]
mod tests;

pub use identifier::Identifier as Id;
pub use reference::Reference as Ref;

#[cfg(feature = "serde")]
pub use serde_update::{erased::ErasedUpdate as ErasedSerDeUpdate, Update as SerDeUpdate};

/// Combine multiple traits into one.
#[macro_export]
macro_rules! combine_traits {
    ($v:vis trait $tn:ident: $($t:ident),+) => {
        $v trait $tn: $($t +)+ {}
        impl<T: $($t +)+> $tn for T {}
    };
}
