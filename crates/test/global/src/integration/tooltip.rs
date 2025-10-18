//! `rimecraft-client-tooltip` integrations.

#![cfg(feature = "tooltip")]

use tooltip::ProvideTooltipTy;

use crate::TestContext;

pub struct OrderedText;



impl ProvideTooltipTy for TestContext {
    const ROW_LENGTH: usize = 170;

    type OrderedText =
        where
            Self: text::ProvideTextTy;
}
