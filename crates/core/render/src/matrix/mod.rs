//! Matrix operations and stacks.

/// Common operations for matrix stacks.
pub trait MatrixStackOp {
    /// The type of entry stored in the stack.
    type Entry;

    /// Pushes a new entry by copying the current top.
    ///
    /// Returns a [`MatrixStackHandle`] that automatically pops when dropped.
    fn push(&mut self) -> MatrixStackHandle<'_, Self::Entry>;

    /// Returns a reference to the top entry.
    fn peek(&self) -> &Self::Entry;

    /// Returns a mutable reference to the top entry.
    fn peek_mut(&mut self) -> &mut Self::Entry;

    /// Returns the current depth (0-indexed).
    fn depth(&self) -> usize;

    /// Returns `true` if only the initial entry exists.
    fn is_empty(&self) -> bool {
        self.depth() == 0
    }
}

/// A stack of transformation matrices with optimized push/pop operations.
///
/// Uses RAII pattern: [`MatrixStack::push()`] returns a handle that automatically pops when dropped.
///
/// # Example
///
/// ```
/// # use rimecraft_render::matrix::{MatrixStack, MatrixStackOp};
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

/// A RAII handle that automatically pops the stack when dropped.
#[derive(Debug)]
pub struct MatrixStackHandle<'a, Entry> {
    stack: &'a mut MatrixStack<Entry>,
}

impl<Entry> Drop for MatrixStackHandle<'_, Entry> {
    #[inline]
    fn drop(&mut self) {
        self.stack.pop();
    }
}

impl<Entry> MatrixStackOp for MatrixStackHandle<'_, Entry>
where
    Entry: Clone,
{
    type Entry = Entry;

    #[inline]
    fn push(&mut self) -> MatrixStackHandle<'_, Self::Entry> {
        self.stack.push()
    }

    #[inline]
    fn peek(&self) -> &Self::Entry {
        self.stack.peek()
    }

    #[inline]
    fn peek_mut(&mut self) -> &mut Self::Entry {
        self.stack.peek_mut()
    }

    #[inline]
    fn depth(&self) -> usize {
        self.stack.depth()
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
}

impl<Entry> MatrixStackOp for MatrixStack<Entry>
where
    Entry: Clone,
{
    type Entry = Entry;

    /// Pushes a new entry by copying the current top.
    ///
    /// Returns a handle that automatically pops when dropped.
    #[inline]
    fn push(&mut self) -> MatrixStackHandle<'_, Self::Entry> {
        let top = self
            .stack
            .last()
            .expect("stack should not be empty")
            .clone();
        self.stack.push(top);
        MatrixStackHandle { stack: self }
    }

    #[inline]
    fn peek(&self) -> &Self::Entry {
        self.stack.last().expect("stack should not be empty")
    }

    #[inline]
    fn peek_mut(&mut self) -> &mut Self::Entry {
        self.stack.last_mut().expect("stack should not be empty")
    }

    #[inline]
    fn depth(&self) -> usize {
        self.stack.len() - 1
    }
}

impl<Entry> Default for MatrixStack<Entry>
where
    Entry: Default + Clone,
{
    fn default() -> Self {
        Self::new(Entry::default())
    }
}

#[cfg(test)]
mod tests;
