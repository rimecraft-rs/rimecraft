//! Advancement related types.

use rimecraft_fmt::Formatting;
use rimecraft_item::{stack::InitAttachments, ItemStack};
use rimecraft_text::Texts;

/// All information about an advancement.\
/// `'r` is registry lifetime.\
/// Generic type `T` is text type, `Id` is identifier
/// type, `Cx` is content type.
///
/// # MCJE Reference
/// `net.minecraft.advancement.Advancement` in yarn.
pub struct Advancement<'r, T, Id, Cx>
where
    T: Texts,
    Cx: InitAttachments<Id>,
{
    pub parent: Option<Id>,
    pub display: Option<DisplayInfo<'r, T, Id, Cx>>,
}

/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementDisplay` in yarn.
pub struct DisplayInfo<'r, T, Id, Cx>
where
    T: Texts,
    Cx: InitAttachments<Id>,
{
    title: T,
    description: T,
    icon: ItemStack<'r, Id, Cx>,
    background: Option<Id>,
    frame: Frame<Id>,
    show_toast: bool,
    announce_to_chat: bool,
    hidden: bool,
    pos: (f32, f32),
}

impl<'r, T, Id, Cx> DisplayInfo<'r, T, Id, Cx>
where
    T: Texts,
    Cx: InitAttachments<Id>,
{
    /// Create a new [`DisplayInfo`].
    pub fn new(
        title: T,
        description: T,
        icon: ItemStack<'r, Id, Cx>,
        background: Option<Id>,
        frame: Frame<Id>,
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

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.pos = (x, y);
    }

    /// Get the title of this advancement.
    pub fn title(&self) -> &T {
        &self.title
    }
}

/// Describes how an advancement will be announced in the chat.\
/// Generic type `Id` represents an identifier type.
///
/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementFrame` in yarn.
#[derive(Debug)]
pub struct Frame<Id> {
    id: Id,
    format: Formatting,
}
