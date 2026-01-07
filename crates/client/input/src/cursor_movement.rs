//! Handles cursor movements.

/// Represents the movement of a cursor.
#[derive(Debug)]
#[non_exhaustive]
pub enum CursorMovement {
    /// Represents an absolute cursor movement.
    Absolute,
    /// Represents a relative cursor movement.
    Relative,
    /// Represents the end of cursor movement.
    End,
}
