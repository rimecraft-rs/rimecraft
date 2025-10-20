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
    stack.push();
    stack.clear();
    assert_eq!(stack.depth(), 0);
    assert_eq!(*stack.peek(), 100);
}

#[test]
fn test_preallocated_capacity() {
    let stack = MatrixStack::<i32>::with_capacity(0, 10);
    assert!(stack.stack.capacity() >= 10);
}

#[test]
fn test_nested_blocks() {
    let mut stack = MatrixStack::new(1);

    {
        let mut h1 = stack.push();
        *h1.peek_mut() += 1; // 2

        {
            let mut h2 = h1.push();
            *h2.peek_mut() *= 3; // 6
        } // h2 pops here

        assert_eq!(*h1.peek(), 2);
    } // h1 pops here

    assert_eq!(*stack.peek(), 1);
}

#[test]
fn test_chain_invocations() {
    let mut stack = MatrixStack::new(1);

    fn render_inner<M>(stack: &mut M) -> i32
    where
        M: MatrixStackOp<Entry = i32>,
    {
        let mut h = stack.push();
        *h.peek_mut() += 5;
        *h.peek()
    }

    fn render<M>(stack: &mut M) -> i32
    where
        M: MatrixStackOp<Entry = i32>,
    {
        let mut h1 = stack.push();
        *h1.peek_mut() += 1;

        let mut h2 = h1.push();
        *h2.peek_mut() *= 3;

        render_inner(&mut h2)
    }

    {
        let mut h = stack.push();
        let res = render(&mut h);
        assert_eq!(res, 11); // ((1 + 1) * 3) + 5
        assert_eq!(*h.peek(), 1); // h is back to original
    }
}
