use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use slotmap::{SlotMap, new_key_type};
use ui::framework::{
    ChildrenProvider, Command, CommandOptimizer, CommandQueue, GenerationalKey, MetaProvider,
    UiStore, UiStoreRead,
};
use ui::{ElementMeta, ProvideUiTy};

use crate::TestContext;

new_key_type! {
    struct TestId;
}

/// Test-local metadata type used by the test store read snapshot. Kept in the
/// test crate so the framework crate doesn't depend on a concrete meta type.
#[derive(Debug, Clone, PartialEq)]
pub struct TestElementMeta<K>
where
    K: Copy + Eq,
{
    pub focused: bool,
    pub parent: Option<K>,
    pub children_count: usize,
}

impl<K> ElementMeta for TestElementMeta<K> where K: Copy + Eq {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TestKey<K>(pub K)
where
    K: Copy + Eq;

impl<K> GenerationalKey for TestKey<K>
where
    K: Copy + Eq,
{
    type Id = K;

    fn generation(&self) -> Self::Id {
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

impl<V> Command<TestContext> for TestCommand<<TestContext as ProvideUiTy>::StoreKey, V>
where
    V: Send + 'static,
{
    fn apply(&self, _store: &mut dyn UiStore<TestContext>) {
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
pub struct SimpleQueue {
    inner: Arc<Mutex<Vec<Box<dyn Command<TestContext>>>>>,
}

impl SimpleQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl CommandQueue<TestContext> for SimpleQueue {
    fn submit(&self, cmd: Box<dyn Command<TestContext>>) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(cmd);
    }

    fn drain_into(&self, out: &mut Vec<Box<dyn Command<TestContext>>>) {
        let mut guard = self.inner.lock().unwrap();
        out.append(&mut *guard);
    }
}

// Small read-only wrapper used for optimizer snapshots.
#[derive(Debug, Clone, Copy)]
pub struct SimpleStoreRead<'a, V> {
    index: &'a HashMap<<TestContext as ProvideUiTy>::StoreKey, TestId>,
    elems: &'a SlotMap<TestId, V>,
    metas: &'a HashMap<
        <TestContext as ProvideUiTy>::StoreKey,
        <TestContext as ProvideUiTy>::ElementMeta,
    >,
}

#[allow(single_use_lifetimes)]
impl<'a, V> UiStoreRead<TestContext> for SimpleStoreRead<'a, V> {
    fn exists(&self, id: <TestContext as ProvideUiTy>::StoreKey) -> bool {
        self.index.contains_key(&id)
    }
}

impl<V> MetaProvider<<TestContext as ProvideUiTy>::StoreKey> for SimpleStoreRead<'_, V> {
    type Meta = TestElementMeta<<TestContext as ProvideUiTy>::StoreKey>;

    fn get_meta(&self, id: <TestContext as ProvideUiTy>::StoreKey) -> Option<&Self::Meta> {
        self.metas.get(&id)
    }
}

impl<V> ChildrenProvider<<TestContext as ProvideUiTy>::StoreKey> for SimpleStoreRead<'_, V> {
    type Iter = <TestContext as ProvideUiTy>::ChildrenIter;

    fn children_of(&self, parent: <TestContext as ProvideUiTy>::StoreKey) -> Self::Iter {
        let mut out = Vec::new();
        for (k, meta) in self.metas.iter() {
            if let Some(p) = meta.parent
                && p == parent
            {
                out.push(*k);
            }
        }
        out.into_iter()
    }
}

/// A minimal in-memory UiStore used by tests. It stores a simple map
/// from id -> integer state and processes `TestCommand` values.
#[derive(Default)]
pub struct TestStore<V> {
    // Commands submitted from main thread
    pending: Vec<Box<dyn Command<TestContext>>>,
    // Commands submitted from other threads
    external: Arc<Mutex<Vec<Box<dyn Command<TestContext>>>>>,
    index: HashMap<<TestContext as ProvideUiTy>::StoreKey, TestId>,
    elems: SlotMap<TestId, V>,
    metas:
        HashMap<<TestContext as ProvideUiTy>::StoreKey, <TestContext as ProvideUiTy>::ElementMeta>,
}

impl<V> TestStore<V> {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            external: Arc::new(Mutex::new(Vec::new())),
            index: HashMap::new(),
            elems: SlotMap::with_key(),
            metas: HashMap::new(),
        }
    }

    pub fn get(&self, key: <TestContext as ProvideUiTy>::StoreKey) -> Option<&V> {
        self.id(key).and_then(|id| self.elems.get(id))
    }

    pub fn get_mut(&mut self, key: <TestContext as ProvideUiTy>::StoreKey) -> Option<&mut V> {
        self.id(key).and_then(move |id| self.elems.get_mut(id))
    }

    fn id(&self, key: <TestContext as ProvideUiTy>::StoreKey) -> Option<TestId> {
        self.index.get(&key).copied()
    }

    fn insert(&mut self, key: <TestContext as ProvideUiTy>::StoreKey, value: V) {
        let id = self.elems.insert(value);
        self.index.insert(key, id);
        // Initialize metadata entry for the new element.
        self.metas.insert(
            key,
            TestElementMeta {
                focused: false,
                parent: None,
                children_count: 0,
            },
        );
    }

    fn remove(&mut self, key: <TestContext as ProvideUiTy>::StoreKey) {
        if let Some(id) = self.index.remove(&key) {
            self.elems.remove(id);
            self.metas.remove(&key);
        }
    }

    pub fn set_focused(&mut self, key: <TestContext as ProvideUiTy>::StoreKey, focused: bool) {
        if let Some(meta) = self.metas.get_mut(&key) {
            meta.focused = focused;
        }
    }

    pub fn set_parent(
        &mut self,
        child: <TestContext as ProvideUiTy>::StoreKey,
        parent: Option<<TestContext as ProvideUiTy>::StoreKey>,
    ) {
        if let Some(old_parent) = self.metas.get(&child).and_then(|m| m.parent)
            && let Some(mp) = self.metas.get_mut(&old_parent)
        {
            mp.children_count = mp.children_count.saturating_sub(1);
        }

        if let Some(meta) = self.metas.get_mut(&child) {
            meta.parent = parent;
        }

        if let Some(new_parent) = parent
            && let Some(np) = self.metas.get_mut(&new_parent)
        {
            np.children_count = np.children_count.saturating_add(1);
        }
    }
}

impl<V> UiStore<TestContext> for TestStore<V>
where
    V: Send + 'static,
{
    fn submit_from_main(&mut self, cmd: Box<dyn Command<TestContext>>) {
        self.pending.push(cmd);
    }

    fn submit_from_other(&self, cmd: Box<dyn Command<TestContext>>) {
        let mut guard = self.external.lock().unwrap();
        guard.push(cmd);
    }

    fn drain_pending(&mut self, out: &mut Vec<Box<dyn Command<TestContext>>>) {
        out.append(&mut self.pending);

        let mut guard = self.external.lock().unwrap();
        out.append(&mut *guard);
    }

    fn apply_batch(&mut self, cmds: Vec<Box<dyn Command<TestContext>>>) {
        for cmd in cmds.into_iter() {
            if let Ok(tc) = cmd
                .into_any()
                .downcast::<TestCommand<<TestContext as ProvideUiTy>::StoreKey, V>>()
            {
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

    fn as_read(&self) -> Box<dyn UiStoreRead<TestContext> + '_> {
        Box::new(SimpleStoreRead {
            index: &self.index,
            elems: &self.elems,
            metas: &self.metas,
        })
    }
}

/// A trivial optimizer used in tests: currently just forwards the commands.
#[derive(Debug, Clone, Copy)]
pub struct TestOptimizer;

impl CommandOptimizer<TestContext> for TestOptimizer {
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<TestContext>>>,
        _store: &dyn UiStoreRead<TestContext>,
    ) -> Vec<Box<dyn Command<TestContext>>> {
        cmds
    }
}
