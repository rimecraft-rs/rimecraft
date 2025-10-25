//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use ui::ProvideUiTy;

use crate::{
    TestContext,
    integration::ui::framework::{TestElementMeta, TestKey},
};

pub mod framework;

impl ProvideUiTy for TestContext {
    type UiEventExt = ();
    type SizeConstraintsExt = ();
    type StoreKey = TestKey<u32>;
    type ElementMeta = TestElementMeta<TestKey<u32>>;
}
