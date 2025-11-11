//! `rimecraft-client-key-bind` integrations.

#![cfg(feature = "key-bind")]

use key_bind::{KeyBindHook, ProvideKeyBindTy};

use crate::TestContext;

impl ProvideKeyBindTy for TestContext {
    type KeyBindExt = EmptyKeyBindExt;
}

/// A blank key bind extension for [`TestContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyKeyBindExt;

impl KeyBindHook<TestContext> for EmptyKeyBindExt {}
