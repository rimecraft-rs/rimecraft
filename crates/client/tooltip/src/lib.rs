//! Minecraft client tooltip components.

use std::fmt::{Debug, Display};

use rimecraft_client_narration::{Narratable, NarrationPart};
use rimecraft_global_cx::GlobalContext;
use rimecraft_text::{ProvideTextTy, Text, iter_text::IterText};

/// Global context for [`Tooltip`].
pub trait ProvideTooltipTy: GlobalContext {
    /// The number of characters per row in the tooltip.
    const ROW_LENGTH: usize;
}

/// Displays a tooltip with text content and optional narration.
pub struct Tooltip<Cx, IText>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    IText: IterText<Cx>,
{
    content: Text<Cx>,
    narration: Option<Text<Cx>>,
    lines: Vec<IText>,
}

impl<Cx, IT> Debug for Tooltip<Cx, IT>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    <Cx as ProvideTextTy>::Content: Debug,
    <Cx as ProvideTextTy>::StyleExt: Debug,
    IT: IterText<Cx> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tooltip")
            .field("content", &self.content)
            .field("narration", &self.narration)
            .field("lines", &self.lines)
            .finish()
    }
}

impl<Cx, IText> Tooltip<Cx, IText>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    IText: IterText<Cx>,
{
    /// Creates a new [`Tooltip`] with the given content and optional narration.
    pub fn new(content: Text<Cx>, narration: Option<Text<Cx>>) -> Self {
        Self {
            content,
            narration,
            lines: Vec::new(),
        }
    }

    /// Creates a new [`Tooltip`] with the given text as both content and narration.
    pub fn of(content: Text<Cx>) -> Self
    where
        <Cx as ProvideTextTy>::Content: Clone,
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        Self::new(content.clone(), Some(content))
    }

    /// Returns the tooltip lines.
    pub fn get_lines(&self) -> &Vec<IText> {
        &self.lines
    }
}

impl<Cx, IText> Narratable for Tooltip<Cx, IText>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    <Cx as ProvideTextTy>::Content: Display,
    IText: IterText<Cx>,
{
    fn append_narrations<B>(&self, builder: &mut B)
    where
        B: rimecraft_client_narration::NarrationMessageBuilder,
    {
        if let Some(narration) = &self.narration {
            builder.put_text::<Cx>(NarrationPart::Hint, narration);
        }
    }
}
