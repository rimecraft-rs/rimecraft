//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use std::{
    cell::RefCell,
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
};

use local_cx::{BaseLocalContext, LocalContext};
use ui::{Element, ProvideUiTy};

use crate::{TestContext, integration::arena::TestArena};

type Arena<'a> = TestArena<RefCell<Box<dyn Element<'a, TestContext>>>>;

pub struct ArenaLocalContext<'a> {
    arena: Arc<Arena<'a>>,
}

impl<'a> ArenaLocalContext<'a> {
    pub fn new(arena: Arc<Arena<'a>>) -> Self {
        Self { arena }
    }
}

impl Debug for ArenaLocalContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArenaLocalContext").finish()
    }
}

impl BaseLocalContext for &ArenaLocalContext<'_> {}

impl<'a> LocalContext<&TestArena<RefCell<Box<dyn Element<'a, TestContext>>>>>
    for &'a ArenaLocalContext<'a>
{
    fn acquire(self) -> &'a TestArena<RefCell<Box<dyn Element<'a, TestContext>>>> {
        &self.arena
    }
}

impl<'a> ProvideUiTy<'a> for TestContext {
    type ElementCell = RefCell<Box<dyn Element<'a, Self>>>;
    type Arena = TestArena<Self::ElementCell>;
    type ArenaLocalContext = &'a ArenaLocalContext<'a>;
}
