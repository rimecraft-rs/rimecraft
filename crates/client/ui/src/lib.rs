//! Minecraft client UI framework.

use std::any::Any;

use rimecraft_client_narration::Narratable;
use rimecraft_keyboard::{KeyState, ProvideKeyboardTy};
use rimecraft_mouse::{ButtonState, MousePos, MouseScroll, ProvideMouseTy};
use rimecraft_render_math::screen::ScreenSize;

use crate::{
    framework::{Command, UiStoreRead},
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
    type ElementMeta: ElementMeta;
    type ChildrenIter: IntoIterator<Item = Self::StoreKey>;
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

pub trait ElementMeta: Clone + Sized {}

pub trait Element<Cx>
where
    Cx: ProvideUiTy,
{
}

/// Read-only, pure-decision event handler for element implementations.
///
/// Implementors should not perform mutations on the store in this method.
/// Instead they return zero-or-more `Command<K>` values which will be
/// collected by the coordinator and applied in a single `apply_batch` call.
pub trait InteractiveElement<Cx>: Element<Cx>
where
    Cx: ProvideUiTy,
{
    /// Decides how to react to `ev` using only the read-only `store` view.
    /// Returns a propagation decision and a list of commands to apply.
    fn handle_event_read(
        &self,
        ev: &dyn Any,
        store: &dyn UiStoreRead<Cx>,
    ) -> (EventPropagation, Vec<Box<dyn Command<Cx>>>) {
        if let Some(ui_ev) = ev.downcast_ref::<UiEvent<'_, Cx>>() {
            self.handle_ui_event_read(ui_ev, store)
        } else {
            (EventPropagation::NotHandled, Vec::new())
        }
    }

    /// Decides how to react to a typed UI event using only the read-only `store` view.
    /// Returns a propagation decision and a list of commands to apply.
    ///
    /// This function should never be called directly; instead, use [`InteractiveElement::handle_event_read`].
    fn handle_ui_event_read(
        &self,
        ev: &UiEvent<'_, Cx>,
        store: &dyn UiStoreRead<Cx>,
    ) -> (EventPropagation, Vec<Box<dyn Command<Cx>>>) {
        let _ = (ev, store);
        (EventPropagation::NotHandled, Vec::new())
    }
}

/// Helper for container implementations: iterate children and invoke a
/// per-child read-time handler. Collects commands returned by children and
/// respects propagation (stops if a child returns `Handled`).
pub fn container_handle_event_read<Cx, F>(
    children: Cx::ChildrenIter,
    ev: &dyn Any,
    store: &dyn UiStoreRead<Cx>,
    mut call_child: F,
) -> (EventPropagation, Vec<Box<dyn Command<Cx>>>)
where
    Cx: ProvideUiTy,
    F: FnMut(
        Cx::StoreKey,
        &dyn Any,
        &dyn UiStoreRead<Cx>,
    ) -> (EventPropagation, Vec<Box<dyn Command<Cx>>>),
{
    let mut out_cmds: Vec<Box<dyn Command<Cx>>> = Vec::new();
    for child in children {
        if !store.exists(child) {
            continue;
        }

        let (prop, mut cmds) = store;
        out_cmds.append(&mut cmds);
        if prop.should_stop() {
            return (EventPropagation::Handled, out_cmds);
        }
    }
    (EventPropagation::NotHandled, out_cmds)
}
