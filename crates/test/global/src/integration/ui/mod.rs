//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use ui::{Element, ProvideUiTy};

use crate::TestContext;

impl ProvideUiTy for TestContext {
    type UiEventExt = ();
    type SizeConstraintsExt = ();
    type ElementIter<'a>
        = Vec<&'a dyn Element<Self>>
    where
        Self: 'a;
}
