//! Advancement related types.

#[cfg(feature = "edcode")]
mod edcode;

mod dbg_impl;

pub mod criterion;

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
    /// See [`DisplayInfo`].
    pub display: Option<DisplayInfo<'r, Cx>>,
}

/// Display-related information.
///
/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementDisplay` in yarn.
pub struct DisplayInfo<'r, Cx>
where
    Cx: AdvancementCx,
{
    pub(crate) title: Text<Cx>,
    pub(crate) description: Text<Cx>,
    pub(crate) icon: ItemStack<'r, Cx>,
    pub(crate) background: Option<Cx::Id>,
    pub(crate) frame: Frame,
    /// Show a notice box at the upper right corner when obtained.
    pub(crate) show_toast: bool,
    /// Send message in chat when obtained.
    pub(crate) announce_to_chat: bool,
    pub(crate) hidden: bool,
    pub(crate) pos: (f32, f32),
}

impl<'r, Cx> DisplayInfo<'r, Cx>
where
    Cx: AdvancementCx,
{
    /// Sets advancement's position.
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.pos = (x, y);
    }

    /// Get the title of this advancement.
    #[inline(always)]
    pub fn title(&self) -> &Text<Cx> {
        &self.title
    }

    /// Get the description of this advancement.
    #[inline(always)]
    pub fn description(&self) -> &Text<Cx> {
        &self.description
    }

    /// Get the icon of tjis advancement.
    #[inline(always)]
    pub fn icon(&self) -> &ItemStack<'r, Cx> {
        &self.icon
    }

    /// Get the background of this advancement.
    #[inline(always)]
    pub fn background(&self) -> Option<&Cx::Id> {
        self.background.as_ref()
    }

    /// Get the frame of this advancement.
    #[inline(always)]
    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// Get advancement's X position within its board.
    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.pos.0
    }

    /// Get advancement's Y position within its board.
    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.pos.1
    }

    /// Get whether show toast when obtained this advancement.
    #[inline(always)]
    pub fn show_toast(&self) -> bool {
        self.show_toast
    }

    /// Get whether announce to chat when obtained this advancement.
    #[inline(always)]
    pub fn announce_to_chat(&self) -> bool {
        self.announce_to_chat
    }

    /// Get whether this advancement is hidden.
    #[inline(always)]
    pub fn hidden(&self) -> bool {
        self.hidden
    }
}

/// Describes how an advancement will be announced in the chat.
/// # MCJE Reference
/// `net.minecraft.advancement.AdvancementFrame` in yarn.
#[cfg_attr(
    feature = "edcode",
    derive(::rimecraft_edcode2::Encode, ::rimecraft_edcode2::Decode)
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
