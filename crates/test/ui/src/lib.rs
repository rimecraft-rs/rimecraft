//! Essential tests for rimecraft-client-ui that require a context.

#![allow(missing_docs)]
#![cfg(test)]

use std::thread;

use test_global::integration::ui::{
    SimpleQueue, TestCommand, TestCommandKind, TestId, TestOptimizer, TestStore,
};
use ui::framework::{CommandQueue, UiStore, run_pipeline};

#[test]
fn pipeline_applies_main_submitted_commands() {
    let mut store = TestStore::new();
    let queue = SimpleQueue::new();
    let opt = TestOptimizer;

    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(1))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(1, 123))));

    run_pipeline::<TestId, _, _, _>(&mut store, &queue, &opt);

    assert_eq!(store.elems.get(&1), Some(&123));
}

#[test]
fn pipeline_applies_queue_submitted_commands() {
    let mut store = TestStore::new();
    let queue = SimpleQueue::new();
    let opt = TestOptimizer;

    queue.submit(Box::new(TestCommand(TestCommandKind::Create(2))));
    queue.submit(Box::new(TestCommand(TestCommandKind::SetState(2, 7))));

    run_pipeline::<TestId, _, _, _>(&mut store, &queue, &opt);

    assert_eq!(store.elems.get(&2), Some(&7));
}

/// Optimizer that prunes earlier SetState commands and keeps only the last
/// SetState per element id.
#[derive(Debug, Clone, Copy)]
struct PruningOptimizer;

impl ui::framework::CommandOptimizer<TestId> for PruningOptimizer {
    fn optimize(
        &self,
        cmds: Vec<Box<dyn ui::framework::Command<TestId>>>,
        _store: &dyn ui::framework::UiStoreRead<TestId>,
    ) -> Vec<Box<dyn ui::framework::Command<TestId>>> {
        use std::collections::HashMap;

        // Record last index of SetState for each id
        let mut last_set: HashMap<TestId, usize> = HashMap::new();
        for (i, cmd) in cmds.iter().enumerate() {
            if let Some(tc) = cmd.as_any().downcast_ref::<TestCommand>()
                && let TestCommandKind::SetState(id, _) = &tc.0
            {
                last_set.insert(*id, i);
            }
        }

        // Keep commands except SetState that are not the last for their id
        cmds.into_iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                if let Some(tc) = cmd.as_any().downcast_ref::<TestCommand>()
                    && let TestCommandKind::SetState(id, _) = &tc.0
                {
                    if last_set.get(id) == Some(&i) {
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
    let mut store = TestStore::new();
    let queue = SimpleQueue::new();
    let opt = PruningOptimizer;

    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(10))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(10, 1))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(10, 2))));

    run_pipeline::<TestId, _, _, _>(&mut store, &queue, &opt);

    assert_eq!(store.elems.get(&10), Some(&2));
}

#[test]
fn remove_then_late_set_state_is_noop() {
    let mut store = TestStore::new();
    let queue = SimpleQueue::new();
    let opt = TestOptimizer;

    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Create(11))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(11, 5))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::Remove(11))));
    store.submit_from_main(Box::new(TestCommand(TestCommandKind::SetState(11, 9))));

    run_pipeline::<TestId, _, _, _>(&mut store, &queue, &opt);

    assert_eq!(store.elems.get(&11), None);
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
            q.submit(Box::new(TestCommand(TestCommandKind::Create(100 + i))));
            q.submit(Box::new(TestCommand(TestCommandKind::SetState(
                100 + i,
                (i as i32) + 1,
            ))));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    run_pipeline::<TestId, _, _, _>(&mut store, &queue, &opt);

    for i in 0..8u64 {
        assert_eq!(store.elems.get(&(100 + i)), Some(&((i as i32) + 1)));
    }
}
