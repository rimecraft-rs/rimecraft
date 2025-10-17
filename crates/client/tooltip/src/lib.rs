//! Minecraft client tooltip components.

use rimecraft_text::{ProvideTextTy, Text, iter_text::IterText};

pub trait ProvideTooltipTy {
    const ROW_LENGTH: usize;
}

pub struct Tooltip<Cx, IT>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    IT: IterText<<Cx as ProvideTextTy>::StyleExt>,
{
    content: Text<Cx>,
    narration: Option<Text<Cx>>,
    lines: Vec<IT>,
}
