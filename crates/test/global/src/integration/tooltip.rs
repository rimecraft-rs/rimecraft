//! `rimecraft-client-tooltip` integrations.

#![cfg(feature = "tooltip")]

use text::{
    ProvideTextTy,
    iter_text::{IterText, IterTextItem},
};
use tooltip::ProvideTooltipTy;

use crate::TestContext;

impl ProvideTooltipTy for TestContext {
    const ROW_LENGTH: usize = 170;

    type OrderedText
        = sealed::EmptyOrderedText<Self>
    where
        Self: ProvideTextTy;
}

mod sealed {
    use std::fmt::Debug;

    use text::{
        ProvideTextTy,
        iter_text::{IterText, IterTextItem},
    };

    pub struct EmptyOrderedText<Cx>
    where
        Cx: ProvideTextTy,
    {
        pub iter_text: Vec<IterTextItem<Cx>>,
    }

    impl<Cx> Debug for EmptyOrderedText<Cx>
    where
        Cx: ProvideTextTy,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("EmptyOrderedText").finish()
        }
    }

    impl<Cx> IterText<Cx> for EmptyOrderedText<Cx>
    where
        Cx: ProvideTextTy + Clone,
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        fn iter_text(&self) -> impl Iterator<Item = IterTextItem<Cx>> + '_ {
            self.iter_text.clone().into_iter()
        }
    }
}
