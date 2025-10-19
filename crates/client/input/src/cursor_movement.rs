//! Handles cursor movements.

/// The movement of a cursor.
#[derive(Debug)]
#[non_exhaustive]
pub enum CursorMovement {
    /// An absolute cursor movement.
    Absolute,
    /// A relative cursor movement.
    Relative,
    /// The end of a cursor movement.
    End,
}
