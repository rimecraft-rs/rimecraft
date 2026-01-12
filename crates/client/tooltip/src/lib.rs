//! Minecraft client tooltip components.

use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use rimecraft_client_narration::{Narratable, NarrationPart};
use rimecraft_global_cx::GlobalContext;
use rimecraft_text::{
    ProvideTextTy, Text,
    ordered::{ErasedOrderedText, OrderedText as _, OrderedTextItem},
};

/// Global context for [`Tooltip`].
pub trait TooltipCx: GlobalContext {
    /// The number of characters per row in the tooltip.
    const ROW_LENGTH: usize;
}

/// Displays a tooltip with text content and optional narration.
pub struct Tooltip<'t, Cx>
where
    Cx: TooltipCx + ProvideTextTy,
{
    content: Arc<Text<Cx>>,
    narration: Option<Arc<Text<Cx>>>,
    lines: Vec<Box<dyn ErasedOrderedText<Cx> + Send + Sync + 't>>,
}

impl<Cx> Debug for Tooltip<'_, Cx>
where
    Cx: TooltipCx + ProvideTextTy,
    <Cx as ProvideTextTy>::Content: Debug,
    <Cx as ProvideTextTy>::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tooltip")
            .field("content", &self.content)
            .field("narration", &self.narration)
            .finish_non_exhaustive()
    }
}

impl<'t, Cx> Tooltip<'t, Cx>
where
    Cx: TooltipCx + ProvideTextTy,
{
    /// Creates a new [`Tooltip`] with the given content and optional narration.
    pub fn new(content: Text<Cx>, narration: Option<Text<Cx>>) -> Self {
        Self {
            content: Arc::new(content),
            narration: narration.map(Arc::new),
            lines: Vec::new(),
        }
    }

    /// Creates a new [`Tooltip`] with the given text as both content and narration.
    pub fn of(content: Text<Cx>) -> Self
    where
        <Cx as ProvideTextTy>::Content: Clone,
        <Cx as ProvideTextTy>::StyleExt: Clone,
    {
        let content = Arc::new(content);
        Self {
            content: content.clone(),
            narration: Some(content),
            lines: Vec::new(),
        }
    }

    /// Returns the tooltip lines.
    pub fn lines(&self) -> &[Box<dyn ErasedOrderedText<Cx> + Send + Sync + 't>] {
        &self.lines
    }

    /// Converts the tooltip lines into a vector of vectors of [`OrderedTextItem`]s.
    pub fn into_items(self) -> Box<[Box<[OrderedTextItem<Cx>]>]>
    where
        Cx: Clone,
    {
        self.lines
            .into_iter()
            .map(|line| line.iter().collect())
            .collect()
    }
}

impl<Cx> From<Text<Cx>> for Tooltip<'_, Cx>
where
    Cx: TooltipCx + ProvideTextTy,
    <Cx as ProvideTextTy>::Content: Clone,
    <Cx as ProvideTextTy>::StyleExt: Clone,
{
    fn from(content: Text<Cx>) -> Self {
        Self::of(content)
    }
}

impl<Cx> Narratable for Tooltip<'_, Cx>
where
    Cx: TooltipCx + ProvideTextTy,
    <Cx as ProvideTextTy>::Content: Display,
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
