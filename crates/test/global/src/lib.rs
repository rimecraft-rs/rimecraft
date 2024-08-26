//! Global context for testing purposes.

use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, LazyLock},
    thread::ThreadId,
};

use global_cx::{
    nbt::{ReadNbt, UpdateNbt, WriteNbt},
    GlobalContext, ProvideIdTy, ProvideNbtTy, ProvideVersionTy,
};
use parking_lot::Mutex;

mod identifier;
pub mod pool;

/// Integration with several Rimecraft crates.
pub mod integration {
    pub mod component;
    mod registry;
}

pub use identifier::Id;

/// The global context.
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum TestContext {}

unsafe impl GlobalContext for TestContext {}

impl ProvideIdTy for TestContext {
    type Id = Id;
}

impl ProvideVersionTy for TestContext {
    type Version = String;
}

/// A integer array.
#[derive(Debug)]
pub struct NbtIntArray(fastnbt::IntArray);

impl From<Box<[i32]>> for NbtIntArray {
    fn from(array: Box<[i32]>) -> Self {
        Self(fastnbt::IntArray::new(array.into()))
    }
}

impl From<NbtIntArray> for Box<[i32]> {
    fn from(array: NbtIntArray) -> Self {
        array.0.into_inner().into()
    }
}

/// A long array.
#[derive(Debug)]
pub struct NbtLongArray(fastnbt::LongArray);

impl From<Box<[i64]>> for NbtLongArray {
    fn from(array: Box<[i64]>) -> Self {
        Self(fastnbt::LongArray::new(array.into()))
    }
}

impl From<NbtLongArray> for Box<[i64]> {
    fn from(array: NbtLongArray) -> Self {
        array.0.into_inner().into()
    }
}

impl ProvideNbtTy for TestContext {
    // Because the function `compound_to_deserializer` returns a `impl Deserializer<'_>`, we need to
    // use a value type here, instead of a hash map.
    type Compound = fastnbt::Value;

    type IntArray = NbtIntArray;

    type LongArray = NbtLongArray;

    fn compound_to_deserializer(compound: &Self::Compound) -> impl serde::Deserializer<'_> {
        compound
    }
}

impl<T> WriteNbt<T> for TestContext
where
    T: serde::Serialize,
{
    fn write_nbt<W>(value: T, writer: W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        fastnbt::to_writer(writer, &value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl<T> ReadNbt<T> for TestContext
where
    T: serde::de::DeserializeOwned,
{
    fn read_nbt<R>(reader: R) -> Result<T, std::io::Error>
    where
        R: std::io::Read,
    {
        fastnbt::from_reader(reader).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl<T> UpdateNbt<T> for TestContext
where
    T: for<'de> serde_update::Update<'de> + ?Sized,
{
    fn update_nbt<R>(value: &mut T, reader: R) -> Result<(), std::io::Error>
    where
        R: std::io::Read,
    {
        serde_update::Update::update(
            value,
            &mut fastnbt::de::Deserializer::from_reader(reader, fastnbt::DeOpts::new()),
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// An unique identifier for an unit test.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TestId(u64);

static TEST_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

static TESTS: LazyLock<Mutex<HashMap<ThreadId, TestId>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl TestId {
    /// Get the test ID of the current thread.
    ///
    /// If the test ID is not set for this thread, a new one will be generated.
    /// See [`announce`] for setting the test ID manually.
    pub fn current() -> Self {
        let thread_id = std::thread::current().id();
        let mut tests = TESTS.lock();
        if let Some(test_id) = tests.get(&thread_id) {
            *test_id
        } else {
            let test_id = TestId(TEST_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
            tests.insert(thread_id, test_id);
            test_id
        }
    }

    /// Announce the test of the current thread.
    pub fn capture(self) {
        let thread_id = std::thread::current().id();
        let mut tests = TESTS.lock();
        tests.insert(thread_id, self);
    }
}

#[cfg(test)]
mod tests;
