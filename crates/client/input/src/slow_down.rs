//! Defines slow down by a factor.

/// Represents the tickable component.
pub trait SlowDownTickable {
    /// Performs a tick operation with the specified [`SlowDown`].
    fn tick(&mut self, slow_down: SlowDown);
}

/// Represents a slow down state.
#[derive(Debug)]
#[non_exhaustive]
pub enum SlowDown {
    /// Slows down with a factor.
    Yes(f32),
    /// No slow down available.
    No,
}
