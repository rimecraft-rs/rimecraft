//! `rimecraft-client-tooltip` integrations.

#![cfg(feature = "tooltip")]

use tooltip::TooltipCx;

use crate::TestContext;

impl TooltipCx for TestContext {
    const ROW_LENGTH: usize = 170;
}
