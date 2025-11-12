//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use ui::{Element, ProvideUiTy};

use crate::TestContext;

impl ProvideUiTy for TestContext {
    type UiEventExt = EmptyUiEventExt;
    type SizeConstraintsExt = EmptySizeConstraintsExt;
    type PositionConstraintsExt = EmptyPositionConstraintsExt;
    type ElementIter<'a>
        = Vec<&'a dyn Element<Self>>
    where
        Self: 'a;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyUiEventExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptySizeConstraintsExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyPositionConstraintsExt;
