//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use ui::{Element, ProvideUiTy, layout::engine::DefaultLayoutEngine};

use crate::TestContext;

impl ProvideUiTy for TestContext {
    type UiEventExt = EmptyUiEventExt;
    type SizeConstraintsExt = EmptySizeConstraintsExt;
    type PositionConstraintsExt = EmptyPositionConstraintsExt;
    type LayoutMeasurementsExt = EmptyLayoutMeasurementsExt;
    type ElementIter<'a>
        = Vec<&'a dyn Element<Self>>
    where
        Self: 'a;
    type LayoutEngine = DefaultLayoutEngine<Self>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyUiEventExt;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptySizeConstraintsExt;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyPositionConstraintsExt;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyLayoutMeasurementsExt;
