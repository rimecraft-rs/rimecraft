use std::{hash::Hash, sync::Arc, thread};

use test_global::integration::ui::framework::{
    SimpleQueue, TestCommand, TestCommandKind, TestKey, TestOptimizer, TestStore,
};
use ui::framework::{
    Command, CommandOptimizer, CommandQueue, UiHandle, UiStore, UiStoreRead, run_pipeline,
};

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
