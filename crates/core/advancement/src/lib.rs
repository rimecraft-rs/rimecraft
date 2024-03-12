//! Advancement related types.

use std::fmt::Debug;

use rimecraft_item::{stack::ItemStackCx, ItemStack};
use rimecraft_text::{ProvideTextTy, Text};

/// Global context for [`Advancement`].
pub trait AdvancementCx: ProvideTextTy + ItemStackCx {}
impl<T> AdvancementCx for T where T: ProvideTextTy + ItemStackCx {}

/// All information about an advancement.\
/// `'r` is registry lifetime.\
/// Generic type `Cx` is context type.
///
/// # MCJE Reference
/// `net.minecraft.advancement.Advancement` in yarn.
pub struct Advancement<'r, Cx>
where
    Cx: AdvancementCx,
{
    /// Parent advancement.
    pub parent: Option<Cx::Id>,
    pub display: Option<DisplayInfo<'r, Cx>>,
}

/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementDisplay` in yarn.
pub struct DisplayInfo<'r, Cx>
where
    Cx: ItemStackCx + ProvideTextTy,
{
    title: Text<Cx>,
    description: Text<Cx>,
    icon: ItemStack<'r, Cx>,
    background: Option<Cx::Id>,
    frame: Frame,
    show_toast: bool,
    announce_to_chat: bool,
    hidden: bool,
    pos: (f32, f32),
}

impl<'r, Cx> DisplayInfo<'r, Cx>
where
    Cx: AdvancementCx,
{
    /// Create a new [`DisplayInfo`].
    pub fn new(
        title: Text<Cx>,
        description: Text<Cx>,
        icon: ItemStack<'r, Cx>,
        background: Option<Cx::Id>,
        frame: Frame,
        show_toast: bool,
        announce_to_chat: bool,
        hidden: bool,
    ) -> Self {
        Self {
            title,
            description,
            icon,
            background,
            frame,
            show_toast,
            announce_to_chat,
            hidden,
            pos: (0., 0.),
        }
    }

    /// Sets advancement's position.
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.pos = (x, y);
    }

    /// Get the title of this advancement.
    pub fn title(&self) -> &Text<Cx> {
        &self.title
    }
}

/// Describes how an advancement will be announced in the chat.\
/// Generic type `Id` represents an identifier type.
///
/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementFrame` in yarn.
#[derive(Debug)]
#[non_exhaustive]
pub enum Frame {
    /// Regular advancement.
    Task,
    /// A hard advancement, sometimes hidden.
    Challenge,
    /// Regular advancement.
    Goal,
}
