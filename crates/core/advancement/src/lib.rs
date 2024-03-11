//! Advancement related types.

use std::fmt::Debug;

use rimecraft_global_cx::ProvideIdTy;
use rimecraft_item::{stack::ItemStackCx, ItemStack};
use rimecraft_text::{ProvideTextTy, Text};

/// All information about an advancement.\
/// `'r` is registry lifetime.\
/// Generic type `T` is text type, `Id` is identifier type, `Cx` is context type.
///
/// # MCJE Reference
/// `net.minecraft.advancement.Advancement` in yarn.
pub struct Advancement<'r, T, Id, Cx>
where
    T: ProvideTextTy,
    Id: ProvideIdTy,
    Cx: ItemStackCx,
{
    /// Parent advancement.
    pub parent: Option<Id>,
    pub display: Option<DisplayInfo<'r, T, Id, Cx>>,
}

/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementDisplay` in yarn.
pub struct DisplayInfo<'r, T, Id, Cx>
where
    T: ProvideTextTy,
    Id: ProvideIdTy,
    Cx: ItemStackCx,
{
    title: Text<T>,
    description: Text<T>,
    icon: ItemStack<'r, Cx>,
    background: Option<Id>,
    frame: Frame,
    show_toast: bool,
    announce_to_chat: bool,
    hidden: bool,
    pos: (f32, f32),
}

impl<'r, T, Id, Cx> DisplayInfo<'r, T, Id, Cx>
where
    T: ProvideTextTy,
    Id: ProvideIdTy,
    Cx: ItemStackCx,
{
    /// Create a new [`DisplayInfo`].
    pub fn new(
        title: Text<T>,
        description: Text<T>,
        icon: ItemStack<'r, Cx>,
        background: Option<Id>,
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
    pub fn title(&self) -> &Text<T> {
        &self.title
    }
}

/// Describes how an advancement will be announced in the chat.\
/// Generic type `Id` represents an identifier type.
///
/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementFrame` in yarn.
#[derive(Debug)]
pub enum Frame {
    /// Regular advancement.
    Task,
    /// A hard advancement, sometimes hidden.
    Challenge,
    /// Regular advancement.
    Goal,
}
