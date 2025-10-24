//! Minecraft client UI framework.

use rimecraft_arena::Arena;
use rimecraft_cell::Cell;
use rimecraft_client_narration::Narratable;
use rimecraft_keyboard::{KeyState, ProvideKeyboardTy};
use rimecraft_local_cx::LocalContext;
use rimecraft_mouse::{ButtonState, MousePos, MouseScroll, ProvideMouseTy};
use rimecraft_render_math::screen::ScreenRect;

use crate::nav::{NavDirection, WithNavIndex, screen::ScreenRectExt};

pub mod item;
pub mod nav;

pub trait ProvideUiTy<'a>: ProvideKeyboardTy + ProvideMouseTy {
    type ElementCell: Cell<Box<dyn Element<'a, Self>>>;
    type Arena: Arena<Item = Self::ElementCell, Handle: Copy + Eq>;
    type ArenaLocalContext: LocalContext<&'a Self::Arena>;
}

/// The selection state of a UI component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum SelectionState {
    /// The component is hovered but not selected.
    Hovered,
    /// The component is selected and focused.
    Focused,
}

impl SelectionState {
    /// Whether the selection state is `Focused`.
    pub fn is_focused(&self) -> bool {
        matches!(self, SelectionState::Focused)
    }
}

/// The result of an event handling operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)]
pub enum EventPropagation {
    /// The event was handled and should not propagate further.
    Handled,
    /// The event was not handled and should propagate further.
    NotHandled,
}

impl EventPropagation {
    /// Whether the event should stop propagating.
    pub fn should_stop(&self) -> bool {
        matches!(self, EventPropagation::Handled)
    }
}

/// A selectable UI component.
pub trait Selectable: Narratable + WithNavIndex {
    /// Whether this component can be narrated.
    fn can_be_narrated(&self) -> bool {
        true
    }

    /// The [`SelectionState`] of this component, if any.
    fn state(&self) -> Option<SelectionState>;
}
