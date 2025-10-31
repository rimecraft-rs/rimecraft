//! `smallbox` crate integration.

use smallbox::SmallBox;

use crate::GlobalContext;

/// A trait for providing space types for `SmallBox`es.
pub trait ProvideSmallBoxSpaceTy: GlobalContext {
    /// The space type for generic trait objects.
    type GenericTraitObjectSpace;
}

/// A type alias for `SmallBox`ed trait objects.
pub type SmallBoxedTraitObject<T, Cx> =
    SmallBox<T, <Cx as ProvideSmallBoxSpaceTy>::GenericTraitObjectSpace>;
