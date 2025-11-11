//! Minecraft client UI framework.

use std::time::Duration;

use rimecraft_client_narration::Narratable;
use rimecraft_keyboard::{KeyState, ProvideKeyboardTy};
use rimecraft_mouse::{ButtonState, MousePos, MouseScroll, ProvideMouseTy};
use rimecraft_render_math::screen::{ScreenRect, ScreenSize};

use crate::{
    layout::{LayoutMeasurements, position::PositionConstraints, size::SizeConstraints},
    nav::{
        NavDirection, WithNavIndex,
        gui::{GuiNavigation, GuiNavigationPath},
        screen::ScreenRectExt as _,
    },
};

pub mod item;
pub mod layout;
pub mod nav;

/// Provides types and constants for the UI framework.
pub trait ProvideUiTy: ProvideKeyboardTy + ProvideMouseTy {
    /// The extension type for [`UiEvent`].
    type UiEventExt;
    /// The extension type for [`SizeConstraints`].
    type SizeConstraintsExt;
    /// The iterator type to iterate over child elements.
    type ElementIter<'a>: IntoIterator<Item = &'a dyn Element<Self>>
    where
        Self: 'a;

    /// The maximum interval between two clicks to be considered a double-click.
    const MAX_DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(250);
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
        matches!(self, Self::Focused)
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
        matches!(self, Self::Handled)
    }
}

/// A component that can be focused.
pub trait Focusable<Cx>
where
    Cx: ProvideUiTy,
{
    /// Whether this component is currently focused.
    fn is_focused(&self) -> bool;

    /// Sets the focus state of this component.
    fn set_focused(&self, focused: bool);

    /// Focuses this component.
    fn focus(&self) {
        self.set_focused(true);
    }
}

/// A component that can be dragged.
pub trait Draggable<Cx>
where
    Cx: ProvideUiTy,
{
    /// The buttons that are currently dragging this component.
    fn dragging_buttons(&self) -> &[Cx::Button];

    /// Mutable access to the buttons that are currently dragging this component.
    fn dragging_buttons_mut(&mut self) -> &mut Vec<Cx::Button>;

    /// Whether this component is being dragged with the given button. If [`None`] is given, checks if the component is being dragged with any button.
    fn is_dragging(&self, button: Option<Cx::Button>) -> bool {
        match button {
            Some(btn) => self.dragging_buttons().contains(&btn),
            None => !self.dragging_buttons().is_empty(),
        }
    }

    /// Sets the dragging state for the given button.
    fn set_dragging(&mut self, button: Cx::Button, dragging: bool) {
        if dragging {
            if !self.is_dragging(Some(button)) {
                self.dragging_buttons_mut().push(button);
            }
        } else {
            self.dragging_buttons_mut().retain(|&b| b != button);
        }
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

/// An UI element.
pub trait Element<Cx>: Focusable<Cx>
where
    Cx: ProvideUiTy,
{
    /// Handles the propagation of an [`UiEvent`].
    fn handle_ui_event<'a>(&'a self, event: &UiEvent<'a, Cx>) -> EventPropagation {
        let _ = event;
        EventPropagation::NotHandled
    }
}

/// An UI element that works as a container for some children elements.
pub trait ContainerElement<Cx>: Element<Cx>
where
    Cx: ProvideUiTy,
{
    /// The children elements.
    fn children(&self) -> Cx::ElementIter<'_>;

    /// Whether this element contains a child that matches the given pointer.
    fn contains_child(&self, child: &dyn Element<Cx>) -> bool {
        self.children().into_iter().any(|c| std::ptr::eq(c, child))
    }

    /// Whether this element has any focused children.
    fn has_focused_child(&self) -> bool {
        self.children().into_iter().any(|child| child.is_focused())
    }

    /// Sets the focused child to match the given pointer. If [`None`] is given, every child will be blurred. If the given pointer points to an element elsewhere, no operations will be performed.
    fn set_focused_child(&self, child: Option<&dyn Element<Cx>>) {
        if let Some(child) = child {
            if self.contains_child(child) {
                for c in self.children() {
                    c.set_focused(std::ptr::eq(c, child));
                }
            }
        } else {
            self.children()
                .into_iter()
                .for_each(|c| c.set_focused(false));
        }
    }
}

impl<E, Cx> Element<Cx> for E
where
    E: ContainerElement<Cx>,
    Cx: ProvideUiTy,
{
    fn handle_ui_event<'a>(&'a self, event: &UiEvent<'a, Cx>) -> EventPropagation {
        for child in self.children() {
            if child.handle_ui_event(event).should_stop() {
                return EventPropagation::Handled;
            }
        }
        EventPropagation::NotHandled
    }
}

/// An UI element that responds to layout.
pub trait LayoutElement<Cx>: Element<Cx>
where
    Cx: ProvideUiTy,
{
    /// The layout should be updated.
    fn update_layout(
        &self,
        screen_size: ScreenSize,
        parent_size: ScreenSize,
        optimal_size: ScreenSize,
    );

    /// The [`LayoutMeasurements`] of this element.
    fn measurements(&self) -> LayoutMeasurements<ScreenSize>;

    /// The [`SizeConstraints`] of this element.
    fn size_constraints(&self) -> SizeConstraints<Cx::SizeConstraintsExt>;

    /// The [`PositionConstraints`] of this element.
    fn position_constraints(&self) -> PositionConstraints;
}

/// An UI element that supports navigation.
pub trait NavElement<Cx>: Element<Cx> + WithNavIndex
where
    Cx: ProvideUiTy,
{
    /// Returns a [`GuiNavigationPath`] starting from this element according to the given [`GuiNavigation`].
    fn nav_path<N>(&self, nav: N) -> Option<GuiNavigationPath<'_, Cx>>
    where
        N: GuiNavigation,
    {
        drop(nav);
        None
    }

    /// Returns the focused [`GuiNavigationPath`] starting from this element.
    fn focused_path(&self) -> Option<GuiNavigationPath<'_, Cx>>
    where
        Self: Sized,
    {
        self.is_focused().then(|| GuiNavigationPath::new_leaf(self))
    }

    /// The focus rectangle of this element, if any.
    fn focus_rect(&self) -> Option<ScreenRect> {
        None
    }

    /// The focus border of this element in the given direction, if any.
    fn focus_border(&self, direction: NavDirection) -> Option<ScreenRect> {
        self.focus_rect().map(|r| r.border(direction))
    }
}
