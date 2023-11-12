/// Represents a language.
pub trait Lang {
    /// Returns the translation of given translation key and
    /// fallback language if translation is not found.
    fn translation(&self, key: &str, fallback: &str) -> Option<&str>;

    /// Returns the direction of the language.
    fn direction(&self) -> Direction;
}

/// Direction of a language.
///
/// # Examples
///
/// English is [`Self::LeftToRight`],
/// and Arabic is [`Self::RightToLeft`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Direction {
    /// Left to right.
    LeftToRight,
    /// Right to left.
    RightToLeft,
}
