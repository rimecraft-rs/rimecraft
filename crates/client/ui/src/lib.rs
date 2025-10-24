//! Minecraft client UI framework.

use rimecraft_client_narration::Narratable;
use rimecraft_keyboard::{KeyState, ProvideKeyboardTy};
use rimecraft_mouse::{ButtonState, MousePos, MouseScroll, ProvideMouseTy};
use rimecraft_render_math::screen::ScreenSize;

use crate::{
    layout::{LayoutPack, LayoutValue, position::PositionConstraints, size::SizeConstraints},
    nav::WithNavIndex,
};

pub mod framework;
pub mod item;
pub mod layout;
pub mod nav;

pub trait ProvideUiTy: ProvideKeyboardTy + ProvideMouseTy {
    type UiEventExt;
    type SizeConstraintsExt;
    type StoreKey: Copy + Eq;
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

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum UiEvent<'a, Cx>
where
    Cx: ProvideUiTy,
{
    MouseMove(MousePos),
    MouseButton {
        pos: MousePos,
        button: Cx::Button,
        state: ButtonState,
    },
    MouseDrag {
        pos: MousePos,
        delta_pos: MousePos,
        button: Cx::Button,
    },
    MouseScroll {
        pos: MousePos,
        scroll: MouseScroll,
    },
    KeyboardKey {
        key: Cx::Key,
        modifiers: &'a [Cx::Modifier],
        state: KeyState,
    },
    CharInput {
        c: char,
        modifiers: &'a [Cx::Modifier],
    },
    Generic {
        ext: Cx::UiEventExt,
    },
}

pub trait Element<Cx>: Focusable
where
    Cx: ProvideUiTy,
{
}

pub trait ParentElement<Cx>: Element<Cx>
where
    Cx: ProvideUiTy,
{
    fn children(&self) -> Vec<Cx::StoreKey>;
}

pub trait PositionElement<Cx>: Element<Cx>
where
    Cx: ProvideUiTy,
{
    fn position_constraints(&self) -> Option<PositionConstraints>;
}

pub trait LayoutElement<Cx>: Element<Cx>
where
    Cx: ProvideUiTy,
{
    fn measure(
        &self,
        constraints: SizeConstraints<Cx::SizeConstraintsExt>,
    ) -> LayoutPack<Option<LayoutValue>> {
        constraints.maximum_size()
    }

    fn layout(&mut self, size: ScreenSize);
}

pub trait InteractiveElement<Cx>: Element<Cx>
where
    Cx: ProvideUiTy,
{
    /// Handles a UI event.
    fn handle_event(&mut self, event: &UiEvent<'_, Cx>) -> EventPropagation;
}

pub trait ParentInteractiveElement<Cx>: InteractiveElement<Cx>
where
    Cx: ProvideUiTy,
{
}
