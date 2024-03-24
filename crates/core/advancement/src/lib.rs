//! Advancement related types.

#[cfg(feature = "edcode")]
mod edcode;

mod dbg_impl;

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
    frame: Frame,
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
    pub frame: Frame,
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
#[cfg_attr(
    feature = "edcode",
    derive(::rimecraft_edcode::Encode, ::rimecraft_edcode::Decode)
)]
#[derive(Debug, Clone)]
#[non_exhaustive]
#[repr(u8)]
pub enum Frame {
    /// Regular task.
    Task = 0,
    /// A hard challenge, sometimes hidden.
    Challenge = 1,
    /// A regular goal.
    Goal = 2,
}

impl Frame {
    /// Turns [`Frame`] into corresponding [`FrameData`].
    pub fn data(&self) -> FrameData {
        match self {
            Self::Task => FrameData {
                name: "task",
                fmt: Formatting::Green,
            },
            Self::Challenge => FrameData {
                name: "challenge",
                fmt: Formatting::DarkPurple,
            },
            Self::Goal => FrameData {
                name: "goal",
                fmt: Formatting::Green,
            },
        }
    }
}

/// Inner type of [`Frame`].
#[derive(Debug, Clone)]
pub struct FrameData {
    /// Chat announcement text id.
    pub name: &'static str,
    /// Color
    pub fmt: Formatting,
}
