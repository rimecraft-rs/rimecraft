use std::{hash::Hash, sync::Arc, thread};

use std::any::Any;
use test_global::TestContext;
use test_global::integration::ui::framework::{
    SimpleQueue, TestCommand, TestCommandKind, TestKey, TestOptimizer, TestStore,
};
use ui::framework::{
    Command, CommandOptimizer, CommandQueue, UiHandle, UiStore, UiStoreRead, run_pipeline,
};
use ui::{Element, EventPropagation, InteractiveElement};

#[test]
fn pipeline_applies_main_submitted_commands() {
    let mut store = TestStore::new();
    let queue = Arc::new(SimpleQueue::new());
    let opt = TestOptimizer;

    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey("one"),
        0,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey("one"),
        123,
    ))));

    run_pipeline(&mut store, queue.as_ref(), &opt);

    assert_eq!(store.get(TestKey("one")), Some(&123));
}

#[test]
fn pipeline_applies_queue_submitted_commands() {
    let mut store = TestStore::new();
    let queue = Arc::new(SimpleQueue::new());
    let handle = UiHandle::<TestKey<u32>>::new(queue.clone());
    let opt = TestOptimizer;

    handle.submit(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(2),
        0,
    ))));
    handle.submit(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey(2),
        7,
    ))));

    run_pipeline(&mut store, queue.as_ref(), &opt);

    assert_eq!(store.get(TestKey(2)), Some(&7));
}

/// Optimizer that prunes earlier SetState commands and keeps only the last
/// SetState per element id.
#[derive(Debug, Default, Clone, Copy)]
struct PruningOptimizer<V> {
    _marker: std::marker::PhantomData<V>,
}

impl<K, V> CommandOptimizer<K> for PruningOptimizer<V>
where
    K: Copy + Eq + Hash + 'static,
    V: Send + Sync + 'static,
{
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<K>>>,
        _store: &dyn UiStoreRead<K>,
    ) -> Vec<Box<dyn Command<K>>> {
        use std::collections::HashMap;

        // Record last index of SetState for each id
        let mut last_set: HashMap<K, usize> = HashMap::new();
        for (i, cmd) in cmds.iter().enumerate() {
            if let Some(tc) = cmd.as_any().downcast_ref::<TestCommand<K, V>>()
                && let TestCommandKind::SetState(k, _) = &tc.0
            {
                last_set.insert(*k, i);
            }
        }

        // Keep commands except SetState that are not the last for their id
        cmds.into_iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                if let Some(tc) = cmd.as_any().downcast_ref::<TestCommand<K, V>>()
                    && let TestCommandKind::SetState(k, _) = &tc.0
                {
                    if last_set.get(k) == Some(&i) {
                        return Some(cmd);
                    } else {
                        return None;
                    }
                }
                Some(cmd)
            })
            .collect()
    }
}

#[test]
fn pruning_optimizer_keeps_last_set_state() {
    let mut store = TestStore::<_, u32>::new();
    let queue = Arc::new(SimpleQueue::new());
    let opt = PruningOptimizer::<u32>::default();

    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(10),
        0_u32,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey(10),
        1_u32,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey(10),
        2_u32,
    ))));

    run_pipeline(&mut store, queue.as_ref(), &opt);

    assert_eq!(store.get(TestKey(10)), Some(&2));
}

#[test]
fn remove_then_late_set_state_is_noop() {
    let mut store = TestStore::<_, i32>::new();
    let queue = SimpleQueue::new();
    let opt = TestOptimizer;

    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(11),
        0,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey(11),
        5,
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::<_, i32>::Remove(
        TestKey(11),
    ))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(
        TestKey(11),
        9,
    ))));

    run_pipeline(&mut store, &queue, &opt);

    assert_eq!(store.get(TestKey(11)), None);
}

#[test]
fn concurrent_queue_submissions() {
    let mut store = TestStore::new();
    let queue = SimpleQueue::new();
    let opt = TestOptimizer;

    let mut handles = Vec::new();
    for i in 0..8u64 {
        let q = queue.clone();
        handles.push(thread::spawn(move || {
            q.submit(Box::new(TestCommand(TestCommandKind::Create(
                TestKey(100 + i),
                0,
            ))));
            q.submit(Box::new(TestCommand(TestCommandKind::SetState(
                TestKey(100 + i),
                (i as i32) + 1,
            ))));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    run_pipeline(&mut store, &queue, &opt);

    for i in 0..8u64 {
        assert_eq!(store.get(TestKey(100 + i)), Some(&((i as i32) + 1)));
    }
}

#[test]
fn parent_dispatches_child_events() {
    // Setup store and optimizer
    let mut store = TestStore::<_, u32>::new();
    let queue = Arc::new(SimpleQueue::new());
    let opt = TestOptimizer;

    // Create a child element in the store with key 1
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(
        TestKey(1u32),
        0u32,
    ))));
    run_pipeline(&mut store, queue.as_ref(), &opt);

    // A minimal ElementRead implementation: when it sees the string "click"
    // it returns a SetState command for the target id.
    struct Child;

    impl Element<TestContext> for Child {}

    impl InteractiveElement<TestContext> for Child {
        fn handle_event_read(
            &self,
            ev: &dyn Any,
            store_read: &dyn UiStoreRead<TestKey<u32>>,
        ) -> (EventPropagation, Vec<Box<dyn Command<TestKey<u32>>>>) {
            if let Some(s) = ev.downcast_ref::<&'static str>()
                && *s == "click"
            {
                // Only emit command if the element exists in the store
                if store_read.exists(TestKey(1u32)) {
                    let cmd: Box<dyn Command<TestKey<u32>>> =
                        Box::new(TestCommand(TestCommandKind::SetState(TestKey(1u32), 42u32)));
                    return (EventPropagation::Handled, vec![cmd]);
                }
            }
            (EventPropagation::NotHandled, Vec::new())
        }
    }

    let child = Child;

    // Coordinator-like flow: take a read snapshot, ask child what to do,
    // then optimize and apply.
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
