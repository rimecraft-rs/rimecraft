use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use slotmap::{SlotMap, new_key_type};
use ui::framework::{
    Command, CommandOptimizer, CommandQueue, GenerationalKey, UiStore, UiStoreRead,
};

new_key_type! {
    struct TestId;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TestKey<K>(pub K)
where
    K: Copy + Eq;

impl<K> GenerationalKey for TestKey<K>
where
    K: Copy + Eq,
{
    type I = K;

    fn generation(&self) -> Self::I {
        self.0
    }
}

/// A tiny command enum used by tests.
#[derive(Clone, PartialEq, Eq)]
pub enum TestCommandKind<K, V>
where
    K: Copy + Eq,
{
    Create(K, V),
    Remove(K),
    SetState(K, V),
}

/// Boxable command value used by the test queue/store.
#[derive(Clone)]
pub struct TestCommand<K, V>(pub TestCommandKind<K, V>)
where
    K: Copy + Eq;

impl<K, V> Command<K> for TestCommand<K, V>
where
    K: Copy + Eq + Send + 'static,
    V: Send + 'static,
{
    fn apply(&self, _store: &mut dyn UiStore<K>) {
        // The concrete TestStore inspects and applies commands itself
        // Commands are data-only here so apply is a no-op
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Simple thread-safe queue for submitting test commands.
#[derive(Default, Clone)]
pub struct SimpleQueue<K>
where
    K: Copy + Eq + Hash,
{
    inner: Arc<Mutex<Vec<Box<dyn Command<K>>>>>,
}

impl<K> SimpleQueue<K>
where
    K: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<K> CommandQueue<K> for SimpleQueue<K>
where
    K: Copy + Eq + Hash + Send + 'static,
{
    fn submit(&self, cmd: Box<dyn Command<K>>) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(cmd);
    }

    fn drain_into(&self, out: &mut Vec<Box<dyn Command<K>>>) {
        let mut guard = self.inner.lock().unwrap();
        out.append(&mut *guard);
    }
}

// Small read-only wrapper used for optimizer snapshots.
pub struct SimpleStoreRead<'a, K, V>
where
    K: Eq + Hash,
{
    index: &'a HashMap<K, TestId>,
    elems: &'a SlotMap<TestId, V>,
}

#[allow(single_use_lifetimes)]
impl<'a, K, V> UiStoreRead<K> for SimpleStoreRead<'a, K, V>
where
    K: Eq + Hash,
{
    fn exists(&self, id: K) -> bool {
        self.index.contains_key(&id)
    }
}

/// A minimal in-memory UiStore used by tests. It stores a simple map
/// from id -> integer state and processes `TestCommand` values.
#[derive(Default)]
pub struct TestStore<K, V>
where
    K: Copy + Eq + Hash,
{
    // Commands submitted from main thread
    pending: Vec<Box<dyn Command<K>>>,
    // Commands submitted from other threads
    external: Arc<Mutex<Vec<Box<dyn Command<K>>>>>,
    index: HashMap<K, TestId>,
    elems: SlotMap<TestId, V>,
}

impl<K, V> TestStore<K, V>
where
    K: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            external: Arc::new(Mutex::new(Vec::new())),
            index: HashMap::new(),
            elems: SlotMap::with_key(),
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.id(key).and_then(|id| self.elems.get(id))
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.id(key).and_then(move |id| self.elems.get_mut(id))
    }

    fn id(&self, key: K) -> Option<TestId> {
        self.index.get(&key).copied()
    }

    fn insert(&mut self, key: K, value: V) {
        let id = self.elems.insert(value);
        self.index.insert(key, id);
    }

    fn remove(&mut self, key: K) {
        if let Some(id) = self.index.remove(&key) {
            self.elems.remove(id);
        }
    }
}

impl<K, V> UiStore<K> for TestStore<K, V>
where
    K: Copy + Eq + Hash + Send + 'static,
    V: Send + 'static,
{
    fn submit_from_main(&mut self, cmd: Box<dyn Command<K>>) {
        self.pending.push(cmd);
    }

    fn submit_from_other(&self, cmd: Box<dyn Command<K>>) {
        let mut guard = self.external.lock().unwrap();
        guard.push(cmd);
    }

    fn drain_pending(&mut self, out: &mut Vec<Box<dyn Command<K>>>) {
        out.append(&mut self.pending);

        let mut guard = self.external.lock().unwrap();
        out.append(&mut *guard);
    }

    fn apply_batch(&mut self, cmds: Vec<Box<dyn Command<K>>>) {
        for cmd in cmds.into_iter() {
            if let Ok(tc) = cmd.into_any().downcast::<TestCommand<K, V>>() {
                match tc.0 {
                    TestCommandKind::Create(k, v) => self.insert(k, v),
                    TestCommandKind::Remove(k) => {
                        self.remove(k);
                    }
                    TestCommandKind::SetState(k, v) => {
                        if let Some(e) = self.get_mut(k) {
                            *e = v;
                        }
                    }
                }
            }
        }
    }

    fn as_read(&self) -> Box<dyn UiStoreRead<K> + '_> {
        Box::new(SimpleStoreRead {
            index: &self.index,
            elems: &self.elems,
        })
    }
}

/// A trivial optimizer used in tests: currently just forwards the commands.
#[derive(Debug, Clone, Copy)]
pub struct TestOptimizer;

impl<K> CommandOptimizer<K> for TestOptimizer
where
    K: Copy + Eq + Hash + Send + Sync,
{
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<K>>>,
        _store: &dyn UiStoreRead<K>,
    ) -> Vec<Box<dyn Command<K>>> {
        cmds
    }
}
