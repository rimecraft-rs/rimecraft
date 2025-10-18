/// Type of light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)] // this could be exhaustive. can't imagine a new light type.
pub enum LightType {
    /// Sky light.
    Sky,
    /// Block luminance.
    Block,
}
