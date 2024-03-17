//! Advancement related types.

#[cfg(feature = "edcode")]
mod edcode;

use std::{fmt::Debug, marker::PhantomData};

use rimecraft_fmt::Formatting;
use rimecraft_item::{stack::ItemStackCx, ItemStack};
use rimecraft_text::{ProvideTextTy, Text};

/// Global context for [`Advancement`].
pub trait AdvancementCx: ProvideTextTy + ItemStackCx {}

#[cfg(feature = "edcode")]
pub use edcode::AdvancementEdcodeCx;

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
    Cx: AdvancementCx,
{
    title: Text<Cx>,
    description: Text<Cx>,
    icon: ItemStack<'r, Cx>,
    background: Option<Cx::Id>,
    frame: Frame<Cx>,
    /// Send message in chat when obtained.
    show_toast: bool,
    announce_to_chat: bool,
    hidden: bool,
    pos: (f32, f32),
}

/// Use this to construct a new [`DisplayInfo`].
pub struct NewDisplayInfoDescriptor<'r, Cx>
where
    Cx: AdvancementCx,
{
    /// See [`DisplayInfo::title`].
    pub title: Text<Cx>,
    /// See [`DisplayInfo::description`].
    pub description: Text<Cx>,
    /// See [`DisplayInfo::icon`].
    pub icon: ItemStack<'r, Cx>,
    /// See [`DisplayInfo::background`].
    pub background: Option<Cx::Id>,
    /// See [`DisplayInfo::frame`].
    pub frame: Frame<Cx>,
    /// See [`DisplayInfo::show_toast`].
    pub show_toast: bool,
    /// See [`DisplayInfo::announce_to_chat`].
    pub announce_to_chat: bool,
    /// See [`DisplayInfo::hidden`].
    pub hidden: bool,
}

impl<'r, Cx> DisplayInfo<'r, Cx>
where
    Cx: AdvancementCx,
{
    /// Create a new [`DisplayInfo`].
    pub fn new(
        NewDisplayInfoDescriptor {
            title,
            description,
            icon,
            background,
            frame,
            show_toast,
            announce_to_chat,
            hidden,
        }: NewDisplayInfoDescriptor<'r, Cx>,
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

/// Describes how an advancement will be announced in the chat.
/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementFrame` in yarn.
#[derive(Debug)]
pub struct Frame<Cx: AdvancementCx> {
    pub(crate) data: &'static FrameData,
    kim: PhantomData<Cx>,
}

/// Inner type of [`Frame`].
#[derive(Debug)]
pub struct FrameData {
    /// Chat announcement text id.
    pub name: &'static str,
    /// Color
    pub fmt: Formatting,
}

impl<Cx> Debug for DisplayInfo<'_, Cx>
where
    Cx: AdvancementCx + Debug,
    Cx::Content: Debug,
    Cx::Id: Debug,
    Cx::Compound: Debug,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisplayInfo")
            .field("title", &self.title)
            .field("description", &self.description)
            .field("icon", &self.icon)
            .field("background", &self.background)
            .field("frame", &self.frame)
            .field("show_toast", &self.show_toast)
            .field("announce_to_chat", &self.announce_to_chat)
            .field("hidden", &self.hidden)
            .field("pos", &self.pos)
            .finish()
    }
}
