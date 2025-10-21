//! `rimecraft-client-key-bind` integrations.

#![cfg(feature = "key-bind")]

use key_bind::ProvideKeyBindTy;

use crate::TestContext;

impl ProvideKeyBindTy for TestContext {
    type KeyBindExt = ();
}
