//! Minecraft client UI framework.

use rimecraft_client_narration::Narratable;
use rimecraft_keyboard::{KeyState, ProvideKeyboardTy};
use rimecraft_mouse::{ButtonState, MousePos, MouseScroll, ProvideMouseTy};

use crate::nav::WithNavIndex;

pub mod item;
pub mod layout;
pub mod nav;

pub trait ProvideUiTy: ProvideKeyboardTy + ProvideMouseTy {
    type UiEventExt;
    type SizeConstraintsExt;
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

/// A selectable UI component.
pub trait Selectable: Narratable + WithNavIndex {
    /// Whether this component can be narrated.
    fn can_be_narrated(&self) -> bool {
        true
    }

    /// The [`SelectionState`] of this component, if any.
    fn state(&self) -> Option<SelectionState>;
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

pub trait Focusable {
    /// Whether this component is currently focused.
    fn is_focused(&self) -> bool;

    fn set_focused(&mut self, focused: bool);

    fn focus(&mut self) {
        self.set_focused(true);
    }
}

/// Common UI events.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum UiEvent<'a, Cx>
where
    Cx: ProvideUiTy,
{
    /// The mouse was moved.
    MouseMove(MousePos),
    /// A mouse button toggle event.
    MouseButton {
        /// The position of the mouse.
        pos: MousePos,
        /// The button that was toggled.
        button: Cx::Button,
        /// The new state of the button.
        state: ButtonState,
    },
    /// A mouse drag event.
    MouseDrag {
        /// The position of the mouse.
        pos: MousePos,
        /// The change in position of the mouse.
        delta_pos: MousePos,
        /// The button that was toggled.
        button: Cx::Button,
    },
    /// A mouse scroll event.
    MouseScroll {
        /// The position of the mouse.
        pos: MousePos,
        /// The scroll delta.
        scroll: MouseScroll,
    },
    /// A keyboard key event.
    KeyboardKey {
        /// The position of the mouse.
        key: Cx::Key,
        /// The modifiers active during input.
        modifiers: &'a [Cx::Modifier],
        /// The new state of the key.
        state: KeyState,
    },
    /// A character input event.
    CharInput {
        /// The character that was input.
        c: char,
        /// The modifiers active during input.
        modifiers: &'a [Cx::Modifier],
    },
    /// A generic event.
    Generic {
        /// The extension data.
        ext: Cx::UiEventExt,
    },
}

pub trait Element<Cx>
where
    Cx: ProvideUiTy,
{
}
