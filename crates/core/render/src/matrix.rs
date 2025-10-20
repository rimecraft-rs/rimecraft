//! Matrix operations and stacks.

/// Common operations for matrix stacks.
///
/// This trait provides methods for accessing and manipulating transformation matrices
/// in a stack-like structure. It's implemented by both [`MatrixStack`] and
/// [`MatrixStackHandle`] to provide a consistent interface.
pub trait MatrixStackOp {
    /// The type of entry stored in the stack.
    type Entry;

    /// Pushes a new entry onto the stack by cloning the current top entry.
    ///
    /// Returns a [`MatrixStackHandle`] that will automatically pop when dropped,
    /// implementing RAII (Resource Acquisition Is Initialization).
    ///
    /// The new entry will always be a copy of the current top entry, ensuring
    /// that modifications to the new entry don't affect the previous state.
    fn push(&mut self) -> MatrixStackHandle<'_, Self::Entry>;

    /// Returns a reference to the top entry.
    fn peek(&self) -> &Self::Entry;

    /// Returns a mutable reference to the top entry.
    fn peek_mut(&mut self) -> &mut Self::Entry;

    /// Returns the current depth of the stack.
    fn depth(&self) -> usize;

    /// Returns `true` if the stack contains only the initial entry.
    fn is_empty(&self) -> bool {
        self.depth() == 0
    }
}

/// A stack of transformation matrices with optimized push/pop operations.
///
/// This implementation uses a depth field instead of removing entries on pop,
/// avoiding frequent allocations and deallocations. Entries are reused when
/// pushing again, which is common in rendering pipelines.
///
/// # Performance
///
/// - `push()`: O(1) in most cases, only copies when depth reaches capacity
/// - `pop()`: O(1) - just decrements depth
/// - `peek()`: O(1) - direct array access
///
/// # RAII Pattern
///
/// The [`push()`](MatrixStackOp::push) method returns a [`MatrixStackHandle`] that
/// automatically pops the stack when dropped, ensuring proper cleanup.
///
/// ```
/// # use rimecraft_render::matrix::MatrixStack;
/// # use rimecraft_render::matrix::MatrixStackOp;
/// let mut stack = MatrixStack::new(0);
/// {
///     let mut handle = stack.push();
///     *handle.peek_mut() = 42;
/// } // Automatically pops here
/// assert_eq!(*stack.peek(), 0);
/// ```
#[derive(Debug)]
pub struct MatrixStack<Entry> {
    stack: Vec<Entry>,
    depth: usize,
}

/// A RAII handle for a pushed matrix stack entry.
///
/// This handle automatically pops the stack when dropped, ensuring proper cleanup
/// and preventing mismatched push/pop calls.
///
/// # Example
///
/// ```
/// # use rimecraft_render::matrix::{MatrixStack, MatrixStackOp};
/// let mut stack = MatrixStack::new(1);
/// {
///     let mut handle = stack.push();
///     *handle.peek_mut() = 2;
///     assert_eq!(*handle.peek(), 2);
/// } // Automatically pops here
/// assert_eq!(*stack.peek(), 1);
/// ```
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
    /// Pops the top entry from the stack (private implementation).
    ///
    /// This doesn't actually remove the entry from the underlying storage,
    /// just decrements the depth. The entry will be reused on the next push.
    ///
    /// # Panics
    ///
    /// Panics if the stack is empty (depth == 0).
    #[inline]
    fn pop(&mut self) {
        assert!(self.depth > 0, "Cannot pop from an empty matrix stack");
        self.depth -= 1;
    }
}

impl<Entry> MatrixStack<Entry>
where
    Entry: Clone,
{
    /// Creates a new empty [`MatrixStack`] with the given initial entry.
    #[inline]
    pub fn new(initial: Entry) -> Self {
        Self {
            stack: vec![initial],
            depth: 0,
        }
    }

    /// Creates a new [`MatrixStack`] with pre-allocated capacity.
    ///
    /// This can improve performance by avoiding reallocations for deep stacks.
    #[inline]
    pub fn with_capacity(initial: Entry, capacity: usize) -> Self {
        let mut stack = Vec::with_capacity(capacity);
        stack.push(initial);
        Self { stack, depth: 0 }
    }

    /// Clears the stack back to just the initial entry.
    #[inline]
    pub fn clear(&mut self) {
        self.depth = 0;
    }
}

impl<Entry> MatrixStackOp for MatrixStack<Entry>
where
    Entry: Clone,
{
    type Entry = Entry;

    /// Pushes a new entry onto the stack by cloning the current top entry.
    ///
    /// Returns a [`MatrixStackHandle`] that will automatically pop when dropped.
    ///
    /// The new entry will always be a copy of the current top entry, ensuring
    /// that modifications to the new entry don't affect the previous state.
    /// This is crucial for maintaining transformation matrix hierarchies.
    ///
    /// If the stack has reached its capacity, this will grow the stack.
    /// Otherwise, it overwrites the existing entry at the next depth level.
    #[inline]
    fn push(&mut self) -> MatrixStackHandle<'_, Self::Entry> {
        let top = self.stack[self.depth].clone();
        self.depth += 1;

        if self.depth >= self.stack.len() {
            self.stack.push(top);
        } else {
            self.stack[self.depth] = top;
        }

        MatrixStackHandle { stack: self }
    }

    #[inline]
    fn peek(&self) -> &Self::Entry {
        &self.stack[self.depth]
    }

    #[inline]
    fn peek_mut(&mut self) -> &mut Self::Entry {
        &mut self.stack[self.depth]
    }

    #[inline]
    fn depth(&self) -> usize {
        self.depth
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
mod tests {
    use super::*;

    #[test]
    fn test_raii_auto_pop() {
        let mut stack = MatrixStack::new(0);

        let mut handle = stack.push();
        *handle.peek_mut() = 42;
        drop(handle); // Automatically pops

        assert_eq!(*stack.peek(), 0);
    }

    #[test]
    fn test_push_always_copies_top() {
        let mut stack = MatrixStack::new(5);

        let mut h1 = stack.push();
        assert_eq!(*h1.peek(), 5); // Copied from base
        *h1.peek_mut() = 10;
        drop(h1);

        // Push again - should copy current top (5), not reuse old value (10)
        let h2 = stack.push();
        assert_eq!(*h2.peek(), 5);
    }

    #[test]
    fn test_nested_transformations() {
        let mut stack = MatrixStack::new(1);

        let mut h1 = stack.push();
        *h1.peek_mut() *= 2; // 2

        let mut h2 = h1.push();
        *h2.peek_mut() += 3; // 5

        let mut h3 = h2.push();
        *h3.peek_mut() *= 10; // 50

        assert_eq!(*h3.peek(), 50);
        drop(h3);
        assert_eq!(*h2.peek(), 5);
        drop(h2);
        assert_eq!(*h1.peek(), 2);
    }

    #[test]
    fn test_trait_object_usage() {
        fn transform(stack: &mut dyn MatrixStackOp<Entry = i32>) {
            *stack.peek_mut() *= 2;
        }

        let mut stack = MatrixStack::new(5);
        let mut handle = stack.push();
        transform(&mut handle);

        assert_eq!(*handle.peek(), 10);
    }

    #[test]
    fn test_complex_types() {
        let mut stack = MatrixStack::new(vec![1, 2, 3]);

        let mut h1 = stack.push();
        h1.peek_mut().push(4);

        let mut h2 = h1.push();
        h2.peek_mut().push(5);
        assert_eq!(*h2.peek(), vec![1, 2, 3, 4, 5]);

        drop(h2);
        assert_eq!(*h1.peek(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_clear_resets_depth() {
        let mut stack = MatrixStack::new(100);
        let _h1 = stack.push();
        drop(_h1);

        stack.clear();
        assert_eq!(stack.depth(), 0);
        assert_eq!(*stack.peek(), 100);
    }

    #[test]
    fn test_preallocated_capacity() {
        let stack = MatrixStack::<i32>::with_capacity(0, 10);
        assert!(stack.stack.capacity() >= 10);
    }
}
