//! Essential tests for rimecraft-client-ui that require a context.

#![allow(missing_docs)]
#![cfg(test)]

use std::{any::Any, sync::Arc};

use test_global::{
    TestContext,
    integration::ui::framework::{
        SimpleQueue, TestCommand, TestCommandKind, TestKey, TestOptimizer, TestStore,
    },
};
use ui::{
    Element, EventPropagation, InteractiveElement, ProvideUiTy,
    framework::{Command, CommandOptimizer, UiStore, UiStoreRead, run_pipeline},
};

pub mod framework;

#[test]
fn parent_dispatches_child_events() {
    // Setup store and optimizer
    let mut store = TestStore::<u32>::new();
    let queue = Arc::new(SimpleQueue::new());
    let opt = TestOptimizer;

    // Create a child element in the store with key 1
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(1),
        0,
    ))));
    run_pipeline(&mut store, queue.as_ref(), &opt);

    // A minimal ElementRead implementation: when it sees the string "click"
    // it returns a SetState command for the target id
    struct Child;

    impl Element<TestContext> for Child {}

    impl InteractiveElement<TestContext> for Child {
        fn handle_event_read(
            &self,
            ev: &dyn Any,
            store_read: &dyn UiStoreRead<TestContext>,
        ) -> (EventPropagation, Vec<Box<dyn Command<TestContext>>>) {
            if let Some(s) = ev.downcast_ref::<&'static str>()
                && *s == "click"
            {
                // Only emit command if the element exists in the store
                if store_read.exists(TestKey(1)) {
                    let cmd = Box::new(TestCommand(TestCommandKind::SetState(TestKey(1), 42u32)));
                    return (EventPropagation::Handled, vec![cmd]);
                }
            }
            (EventPropagation::NotHandled, Vec::new())
        }
    }

    let child = Child;

    // Coordinator-like flow: take a read snapshot, ask child what to do,
    // then optimize and apply
    let snapshot = store.as_read();
    let (prop, cmds) = child.handle_event_read(&"click", snapshot.as_ref());
    drop(snapshot);

    assert!(prop.should_stop());

    // Run optimizer on the commands produced by elements and apply
    let snapshot2 = store.as_read();
    let optimized = opt.optimize(cmds, snapshot2.as_ref());
    drop(snapshot2);

    store.apply_batch(optimized);

    assert_eq!(store.get(TestKey(1u32)), Some(&42u32));
}

#[test]
fn children_and_meta_queries() {
    let mut store = TestStore::<i32>::new();
    let queue = Arc::new(SimpleQueue::new());
    let opt = TestOptimizer;

    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(10),
        0,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(11),
        0,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(12),
        0,
    ))));

    run_pipeline(&mut store, queue.as_ref(), &opt);

    // parent = 10, children = [11, 12]
    store.set_parent(TestKey(11), Some(TestKey(10)));
    store.set_parent(TestKey(12), Some(TestKey(10)));
    store.set_focused(TestKey(11), true);

    let snapshot = store.as_read();
    let children = snapshot.children_of(TestKey(10)).collect::<Vec<_>>();
    assert!(children.contains(&TestKey(11)));
    assert!(children.contains(&TestKey(12)));

    let meta = snapshot.get_meta(TestKey(11)).expect("meta");
    assert!(meta.focused);
    assert_eq!(meta.parent, Some(TestKey(10)));
    drop(snapshot);
}

#[test]
fn optimizer_prunes_non_focused_set_state() {
    struct FocusOnlyOptimizer;
    impl CommandOptimizer<TestContext> for FocusOnlyOptimizer {
        fn optimize(
            &self,
            cmds: Vec<Box<dyn Command<TestContext>>>,
            store: &dyn UiStoreRead<TestContext>,
        ) -> Vec<Box<dyn Command<TestContext>>> {
            let mut kept = Vec::new();
            for cmd in cmds.into_iter() {
                if let Some(tc) = cmd
                    .as_any()
                    .downcast_ref::<TestCommand<<TestContext as ProvideUiTy>::StoreKey, i32>>()
                    && let TestCommandKind::SetState(k, _) = &tc.0
                {
                    if let Some(meta) = store.get_meta(*k)
                        && meta.focused
                    {
                        kept.push(cmd);
                    }
                } else {
                    kept.push(cmd);
                }
            }
            kept
        }
    }

    let mut store = TestStore::<i32>::new();
    let queue = Arc::new(SimpleQueue::new());

    // Create two elements
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(20),
        0,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(21),
        0,
    ))));

    run_pipeline(&mut store, queue.as_ref(), &TestOptimizer);

    // Focus only 20
    store.set_focused(TestKey(20), true);

    // Submit SetState for both
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey(20),
        7,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey(21),
        9,
    ))));

    run_pipeline(&mut store, queue.as_ref(), &FocusOnlyOptimizer);

    assert_eq!(store.get(TestKey(20)), Some(&7));
    assert_eq!(store.get(TestKey(21)), Some(&0));
}
