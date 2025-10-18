//! `rimecraft-client-tooltip` integrations.

#![cfg(feature = "tooltip")]

use tooltip::ProvideTooltipTy;

use crate::TestContext;

impl ProvideTooltipTy for TestContext {
    const ROW_LENGTH: usize = 170;
}
