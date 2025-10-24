use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ui::framework::{Command, CommandOptimizer, CommandQueue, UiStore, UiStoreRead};

/// Test id type used in the simple test implementations.
pub type TestId = u64;

/// A tiny command enum used by tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestCommandKind {
    Create(TestId),
    Remove(TestId),
    SetState(TestId, i32),
}

/// Boxable command value used by the test queue/store.
#[derive(Debug, Clone)]
pub struct TestCommand(pub TestCommandKind);

impl Command<TestId> for TestCommand {
    fn apply(&self, _store: &mut dyn UiStore<TestId>) {
        // The concrete TestStore inspects and applies commands itself
        // Commands are data-only here so apply is a no-op
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Simple thread-safe queue for submitting test commands.
#[derive(Debug, Default, Clone)]
pub struct SimpleQueue {
    inner: Arc<Mutex<Vec<Box<dyn Command<TestId>>>>>,
}

impl SimpleQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl CommandQueue<TestId> for SimpleQueue {
    fn submit(&self, cmd: Box<dyn Command<TestId>>) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(cmd);
    }

    fn drain_into(&self, out: &mut Vec<Box<dyn Command<TestId>>>) {
        let mut guard = self.inner.lock().unwrap();
        out.append(&mut *guard);
    }
}

// Small read-only wrapper used for optimizer snapshots.
#[derive(Debug)]
#[allow(single_use_lifetimes)]
pub struct SimpleStoreRead<'a> {
    map: &'a HashMap<TestId, i32>,
}

#[allow(single_use_lifetimes)]
impl<'a> UiStoreRead<TestId> for SimpleStoreRead<'a> {
    fn exists(&self, id: TestId) -> bool {
        self.map.contains_key(&id)
    }
}

/// A minimal in-memory UiStore used by tests. It stores a simple map
/// from id -> integer state and processes `TestCommand` values.
#[derive(Debug, Default)]
pub struct TestStore {
    // Commands submitted from main thread
    pending: Vec<Box<dyn Command<TestId>>>,
    // Commands submitted from other threads
    external: Arc<Mutex<Vec<Box<dyn Command<TestId>>>>>,
    // Simple element state map
    pub elems: HashMap<TestId, i32>,
}

impl TestStore {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            external: Arc::new(Mutex::new(Vec::new())),
            elems: HashMap::new(),
        }
    }
}

impl UiStore<TestId> for TestStore {
    fn submit_from_main(&mut self, cmd: Box<dyn Command<TestId>>) {
        self.pending.push(cmd);
    }

    fn submit_from_other(&self, cmd: Box<dyn Command<TestId>>) {
        let mut guard = self.external.lock().unwrap();
        guard.push(cmd);
    }

    fn drain_pending(&mut self, out: &mut Vec<Box<dyn Command<TestId>>>) {
        out.append(&mut self.pending);

        let mut guard = self.external.lock().unwrap();
        out.append(&mut *guard);
    }

    fn apply_batch(&mut self, cmds: Vec<Box<dyn Command<TestId>>>) {
        for cmd in cmds.into_iter() {
            if let Some(tc) = cmd.as_any().downcast_ref::<TestCommand>() {
                match &tc.0 {
                    TestCommandKind::Create(id) => {
                        self.elems.insert(*id, 0);
                    }
                    TestCommandKind::Remove(id) => {
                        self.elems.remove(id);
                    }
                    TestCommandKind::SetState(id, v) => {
                        if let Some(e) = self.elems.get_mut(id) {
                            *e = *v;
                        }
                    }
                }
            }
        }
    }

    fn as_read(&self) -> Box<dyn UiStoreRead<TestId> + '_> {
        Box::new(SimpleStoreRead { map: &self.elems })
    }
}

/// A trivial optimizer used in tests: currently just forwards the commands.
#[derive(Debug, Default, Clone, Copy)]
pub struct TestOptimizer;

impl CommandOptimizer<TestId> for TestOptimizer {
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<TestId>>>,
        _store: &dyn UiStoreRead<TestId>,
    ) -> Vec<Box<dyn Command<TestId>>> {
        cmds
    }
}
