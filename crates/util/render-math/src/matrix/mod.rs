//! Matrix operations and stacks.

use std::ops::{Deref, DerefMut};

/// A stack of transformation matrices.
///
/// Uses RAII pattern: [`MatrixStack::push()`] returns a guard that automatically pops when dropped.
///
/// # Example
///
/// ```
/// # use rimecraft_render::matrix::MatrixStack;
/// let mut stack = MatrixStack::new(0);
/// {
///     let mut handle = stack.push();
///     *handle.peek_mut() = 42;
/// } // Automatically pops
/// assert_eq!(*stack.peek(), 0);
/// ```
#[derive(Debug)]
pub struct MatrixStack<Entry> {
    stack: Vec<Entry>,
}

/// A RAII guard that automatically pops the stack when dropped.
#[derive(Debug)]
pub struct MatrixStackGuard<'a, Entry>(&'a mut MatrixStack<Entry>);

impl<Entry> Drop for MatrixStackGuard<'_, Entry> {
    #[inline]
    fn drop(&mut self) {
        self.0.pop();
    }
}

impl<Entry> Deref for MatrixStackGuard<'_, Entry> {
    type Target = MatrixStack<Entry>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<Entry> DerefMut for MatrixStackGuard<'_, Entry> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<Entry> MatrixStack<Entry> {
    /// Pops the top entry from the stack.
    #[inline]
    fn pop(&mut self) {
        assert!(
            self.stack.len() > 1,
            "Cannot pop from an empty matrix stack"
        );
        self.stack.pop();
    }
}

impl<Entry> MatrixStack<Entry>
where
    Entry: Clone,
{
    /// Creates a new matrix stack with the given initial entry.
    #[inline]
    pub fn new(initial: Entry) -> Self {
        Self {
            stack: vec![initial],
        }
    }

    /// Creates a new matrix stack with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(initial: Entry, capacity: usize) -> Self {
        let mut stack = Vec::with_capacity(capacity);
        stack.push(initial);
        Self { stack }
    }

    /// Clears the stack back to just the initial entry.
    #[inline]
    pub fn clear(&mut self) {
        self.stack.truncate(1);
    }

    /// Pushes a new entry by copying the current top.
    ///
    /// Returns a guard that automatically pops when dropped.
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn push(&mut self) -> MatrixStackGuard<'_, Entry> {
        let top = self
            .stack
            .last()
            .expect("stack should not be empty")
            .clone();
        self.stack.push(top);
        MatrixStackGuard(self)
    }

    /// Returns a reference to the top entry.
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn peek(&self) -> &Entry {
        self.stack.last().expect("stack should not be empty")
    }

    /// Returns a mutable reference to the top entry.
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn peek_mut(&mut self) -> &mut Entry {
        self.stack.last_mut().expect("stack should not be empty")
    }

    /// Returns the current depth (0-indexed).
    #[inline]
    pub fn depth(&self) -> usize {
        self.stack.len() - 1
    }

    /// Returns `true` if only the initial entry exists.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.depth() == 0
    }
}

impl<Entry> Default for MatrixStack<Entry>
where
    Entry: Default + Clone,
{
    #[inline]
    fn default() -> Self {
        Self::new(Entry::default())
    }
}

#[cfg(test)]
mod tests;
