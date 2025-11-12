//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

use ui::{Draggable, Element, Focusable, ProvideUiTy, layout::engine::DefaultLayoutEngine};

use crate::{TestContext, integration::mouse::TestButton};

impl ProvideUiTy for TestContext {
    type UiEventExt = EmptyUiEventExt;
    type SizeConstraintsExt = EmptySizeConstraintsExt;
    type PosConstraintsExt = EmptyPosConstraintsExt;
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
pub struct EmptyPosConstraintsExt;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyLayoutMeasurementsExt;

pub struct TestElement {
    pub is_focused: AtomicBool,
    pub dragging_buttons: RefCell<Vec<TestButton>>,
}

impl Focusable<TestContext> for TestElement {
    fn is_focused(&self) -> bool {
        self.is_focused.load(Ordering::SeqCst)
    }

    fn set_focused(&self, focused: bool) {
        self.is_focused.store(focused, Ordering::SeqCst);
    }
}

impl Draggable<TestContext> for TestElement {
    fn dragging_buttons(&self) -> Vec<TestButton> {
        self.dragging_buttons.borrow().clone()
    }

    fn set_dragging_buttons(&self, buttons: Vec<TestButton>) {
        *self.dragging_buttons.borrow_mut() = buttons;
    }
}

impl Element<TestContext> for TestElement {}
