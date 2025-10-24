//! `rimecraft-client-ui` integrations.

#![cfg(feature = "ui")]

use std::{cell::RefCell, sync::Arc};

use local_cx::{BaseLocalContext, LocalContext};
use ui::{Element, ProvideUiTy};

use crate::{TestContext, integration::arena::TestArena};

pub struct ArenaLocalContext<'a> {
    arena: &'a Arc<TestArena<RefCell<Box<dyn Element<'a, TestContext>>>>>,
}

impl BaseLocalContext for &ArenaLocalContext<'_> {}

impl<'g> LocalContext<&TestArena<RefCell<Box<dyn Element<'g, TestContext>>>>>
    for &ArenaLocalContext<'g>
{
    fn acquire(self) -> &'g TestArena<RefCell<Box<dyn Element<'g, TestContext>>>> {
        self.arena
    }
}

impl<'g> ProvideUiTy<'g> for TestContext {
    type ElementCell = RefCell<Box<dyn Element<'g, Self>>>;
    type Arena = TestArena<Self::ElementCell>;
    type ArenaLocalContext = &'g ArenaLocalContext<'g>;
}
