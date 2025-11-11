//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use ui::ProvideUiTy;

use crate::TestContext;

impl ProvideUiTy for TestContext {
    type UiEventExt = ();
    type SizeConstraintsExt = ();
}
