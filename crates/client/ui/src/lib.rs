//! Minecraft client UI framework.

use rimecraft_client_narration::Narratable;
use rimecraft_keyboard::{KeyState, ProvideKeyboardTy};
use rimecraft_mouse::{ButtonState, MousePos, MouseScroll, ProvideMouseTy};
use rimecraft_render_math::screen::ScreenRect;

use crate::nav::{NavDirection, WithNavIndex, screen::ScreenRectExt};

pub mod item;
pub mod nav;

pub trait ProvideUiTy: ProvideKeyboardTy + ProvideMouseTy {
    type FocusWriteGuard<'a>: std::ops::DerefMut<Target = bool> + 'a
    where
        Self: 'a;

    type FocusReadGuard<'a>: std::ops::Deref<Target = bool> + 'a
    where
        Self: 'a;
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

/// A UI component that can be focused.
///
/// The reason to have a `Cx` generic parameter is to allow implementations like `impl<T, Cx> Focusable<Cx> for T where T: Element<Cx>`.
pub trait Focusable<'a, Cx>
where
    Cx: ProvideUiTy,
{
    fn focused(&'a self) -> Cx::FocusReadGuard<'a>;

    fn focused_mut(&'a self) -> Cx::FocusWriteGuard<'a>;

    /// Whether this component is focused.
    fn is_focused(&'a self) -> bool {
        *self.focused()
    }

    /// Sets whether this component is focused.
    fn set_focused(&'a self, focused: bool) {
        *self.focused_mut() = focused;
    }

    /// Focuses this component.
    fn focus(&'a self) {
        self.set_focused(true);
    }
}

/// A UI element that can handle input events.
pub trait Element<'a, Cx>: WithNavIndex + Focusable<'a, Cx>
where
    Cx: ProvideUiTy,
{
    /// Handles mouse movement events.
    fn on_mouse_move(&mut self, pos: MousePos) {
        let _ = pos;
    }

    /// Handles mouse button events.
    fn on_mouse_button(
        &mut self,
        pos: MousePos,
        button: <Cx as ProvideMouseTy>::Button,
        state: ButtonState,
    ) -> EventPropagation {
        drop((pos, button, state));
        EventPropagation::NotHandled
    }

    /// Handles mouse dragged events.
    fn on_mouse_drag(
        &mut self,
        pos: MousePos,
        delta_pos: MousePos,
        button: <Cx as ProvideMouseTy>::Button,
    ) -> EventPropagation {
        drop((pos, delta_pos, button));
        EventPropagation::NotHandled
    }

    /// Handles mouse scroll events.
    fn on_mouse_scroll(&mut self, pos: MousePos, scroll: MouseScroll) -> EventPropagation {
        let _ = (pos, scroll);
        EventPropagation::NotHandled
    }

    /// Handles keyboard key events.
    fn on_keyboard_key(
        &mut self,
        key: <Cx as ProvideKeyboardTy>::Key,
        modifiers: &[<Cx as ProvideKeyboardTy>::Modifier],
        state: KeyState,
    ) -> EventPropagation {
        drop((key, modifiers, state));
        EventPropagation::NotHandled
    }

    /// Handles character typing events.
    fn on_char_type(
        &mut self,
        c: char,
        modifiers: &[<Cx as ProvideKeyboardTy>::Modifier],
    ) -> EventPropagation {
        let _ = (c, modifiers);
        EventPropagation::NotHandled
    }

    /// Whether the given mouse position is within this element.
    fn contains_cursor(&self, pos: MousePos) -> bool {
        let _ = pos;
        false
    }

    /// The navigation bounds of this element.
    fn navigation_bounds(&self) -> Option<ScreenRect> {
        None
    }

    /// The navigation border of this element in the given direction.
    fn navigation_border(&self, direction: NavDirection) -> Option<ScreenRect> {
        self.navigation_bounds().map(|r| r.border(direction))
    }
}

/// A UI element that can have child elements.
pub trait ParentElement<'a, Cx>: Element<'a, Cx>
where
    Cx: ProvideUiTy,
{
    /// The child elements of this parent element.
    fn children(&self) -> &[Box<dyn Element<'a, Cx>>];

    /// The mutable child elements of this parent element.
    fn children_mut(&mut self) -> &mut [Box<dyn Element<'a, Cx>>];

    /// Finds the first child element that is hovered by the given mouse position.
    fn hovered_child(&self, pos: MousePos) -> Option<&dyn Element<'a, Cx>> {
        self.children()
            .iter()
            .find(|child| child.contains_cursor(pos))
            .map(|v| &**v)
    }

    /// Finds the first child element that is hovered by the given mouse position, in mutable form.
    fn hovered_child_mut(&mut self, pos: MousePos) -> Option<&mut Box<dyn Element<'a, Cx>>> {
        self.children_mut()
            .iter_mut()
            .find(|child| child.contains_cursor(pos))
    }

    /// Finds the first focused child element.
    fn focused_child(&self) -> Option<&dyn Element<'a, Cx>> {
        self.children()
            .iter()
            .find(|child| child.is_focused())
            .map(|v| &**v)
    }

    /// Finds the first focused child element, in mutable form.
    fn focused_child_mut(&mut self) -> Option<&mut Box<dyn Element<'a, Cx>>> {
        self.children_mut()
            .iter_mut()
            .find(|child| child.is_focused())
    }

    /// Finds the index of the first child element that is equal to the given element.
    fn child_index(&self, child: &dyn Element<'a, Cx>) -> Option<usize> {
        self.children()
            .iter()
            .position(|c| std::ptr::eq(c.as_ref(), child))
    }

    /// The buttons currently being dragged.
    fn dragging_buttons(&self) -> &[Cx::Button];

    /// The buttons currently being dragged, in mutable form.
    fn dragging_buttons_mut(&mut self) -> &mut Vec<Cx::Button>;
}

pub trait ParentElementFocusableImpl<'a, Cx>: ParentElement<'a, Cx>
where
    Cx: ProvideUiTy,
{
    fn focused(&'a self) -> Cx::FocusReadGuard<'a>;

    fn focused_mut(&'a self) -> Cx::FocusWriteGuard<'a>;

    fn is_focused(&'a self) -> bool {
        self.focused_child().is_some()
    }

    fn set_focused(&self, focused: bool) {
        if focused {
            if self.focused_child().is_none()
                && let Some(first_child) = self.children().first()
            {
                first_child.focus();
            }
        } else if let Some(focused_child) = self.focused_child() {
            *focused_child.focused_mut() = false;
        }
    }
}

pub trait ParentElementFocusableExt<'a, Cx>: ParentElementFocusableImpl<'a, Cx>
where
    Cx: ProvideUiTy,
{
    fn focused(&'a self) -> Cx::FocusReadGuard<'a>;

    fn focused_mut(&'a self) -> Cx::FocusWriteGuard<'a>;
}

impl<'a, T, Cx> ParentElementFocusableImpl<'a, Cx> for T
where
    T: ParentElementFocusableExt<'a, Cx> + ?Sized,
    Cx: ProvideUiTy,
    Cx::FocusReadGuard<'a>: From<bool>,
{
    fn focused(&'a self) -> Cx::FocusReadGuard<'a> {
        <Self as ParentElementFocusableExt<_>>::focused(self)
    }

    fn focused_mut(&'a self) -> Cx::FocusWriteGuard<'a> {
        <Self as ParentElementFocusableExt<_>>::focused_mut(self)
    }
}

impl<'a, T, Cx> Focusable<'a, Cx> for T
where
    T: ParentElementFocusableExt<'a, Cx> + ?Sized,
    Cx: ProvideUiTy,
{
    fn focused(&'a self) -> <Cx as ProvideUiTy>::FocusReadGuard<'a> {
        <Self as ParentElementFocusableExt<_>>::focused(self)
    }

    fn focused_mut(&'a self) -> <Cx as ProvideUiTy>::FocusWriteGuard<'a> {
        <Self as ParentElementFocusableExt<_>>::focused_mut(self)
    }

    fn is_focused(&'a self) -> bool {
        <Self as ParentElementFocusableImpl<_>>::is_focused(self)
    }

    fn set_focused(&'a self, focused: bool) {
        <Self as ParentElementFocusableImpl<_>>::set_focused(self, focused)
    }
}

pub trait ParentElementImpl<'a, Cx>: ParentElement<'a, Cx>
where
    Cx: ProvideUiTy,
    <Cx as ProvideMouseTy>::Button: PartialEq + Clone,
{
    fn on_mouse_button(
        &mut self,
        pos: MousePos,
        button: Cx::Button,
        state: ButtonState,
    ) -> EventPropagation {
        match self.hovered_child(pos).and_then(|c| self.child_index(c)) {
            Some(index) => match state {
                ButtonState::Pressed => {
                    let propagation = Element::on_mouse_button(
                        self.children_mut()[index].as_mut(),
                        pos,
                        button.clone(),
                        state,
                    );
                    match propagation {
                        EventPropagation::Handled => {}
                        EventPropagation::NotHandled => {
                            self.set_focused(false);
                            self.children_mut()[index].focus();
                            self.dragging_buttons_mut().push(button);
                        }
                    };
                    propagation
                }
                ButtonState::Idle => {
                    if self.dragging_buttons().contains(&button) {
                        self.dragging_buttons_mut().retain(|b| b != &button);
                        self.focused_child_mut()
                            .map_or(EventPropagation::NotHandled, |child| {
                                Element::on_mouse_button(child.as_mut(), pos, button, state)
                            })
                    } else {
                        EventPropagation::NotHandled
                    }
                }
                _ => Element::on_mouse_button(
                    self.children_mut()[index].as_mut(),
                    pos,
                    button,
                    state,
                ),
            },
            None => EventPropagation::NotHandled,
        }
    }

    fn on_mouse_drag(
        &mut self,
        pos: MousePos,
        delta_pos: MousePos,
        button: Cx::Button,
    ) -> EventPropagation {
        if self.dragging_buttons().contains(&button) {
            self.focused_child_mut()
                .map_or(EventPropagation::NotHandled, |child| {
                    Element::on_mouse_drag(child.as_mut(), pos, delta_pos, button)
                })
        } else {
            EventPropagation::NotHandled
        }
    }

    fn on_mouse_scroll(&mut self, pos: MousePos, scroll: MouseScroll) -> EventPropagation {
        match self.hovered_child_mut(pos) {
            Some(child) => Element::on_mouse_scroll(child.as_mut(), pos, scroll),
            None => EventPropagation::NotHandled,
        }
    }

    fn on_keyboard_key(
        &mut self,
        key: Cx::Key,
        modifiers: &[Cx::Modifier],
        state: KeyState,
    ) -> EventPropagation {
        match self.focused_child_mut() {
            Some(child) => Element::on_keyboard_key(child.as_mut(), key, modifiers, state),
            None => EventPropagation::NotHandled,
        }
    }

    fn on_char_type(&mut self, c: char, modifiers: &[Cx::Modifier]) -> EventPropagation {
        match self.focused_child_mut() {
            Some(child) => Element::on_char_type(child.as_mut(), c, modifiers),
            None => EventPropagation::NotHandled,
        }
    }
}

pub trait ParentElementExt<'a, Cx>: ParentElementImpl<'a, Cx>
where
    Cx: ProvideUiTy,
    <Cx as ProvideMouseTy>::Button: PartialEq + Clone,
{
}

impl<'a, T, Cx> ParentElementImpl<'a, Cx> for T
where
    T: ParentElementExt<'a, Cx> + ?Sized,
    Cx: ProvideUiTy,
    <Cx as ProvideMouseTy>::Button: PartialEq + Clone,
{
}

impl<'a, T, Cx> Element<'a, Cx> for T
where
    T: ParentElementExt<'a, Cx> + ?Sized,
    Cx: ProvideUiTy,
    <Cx as ProvideMouseTy>::Button: PartialEq + Clone,
{
    fn on_mouse_button(
        &mut self,
        pos: MousePos,
        button: Cx::Button,
        state: ButtonState,
    ) -> EventPropagation {
        <Self as ParentElementImpl<_>>::on_mouse_button(self, pos, button, state)
    }

    fn on_mouse_drag(
        &mut self,
        pos: MousePos,
        delta_pos: MousePos,
        button: Cx::Button,
    ) -> EventPropagation {
        <Self as ParentElementImpl<_>>::on_mouse_drag(self, pos, delta_pos, button)
    }

    fn on_mouse_scroll(&mut self, pos: MousePos, scroll: MouseScroll) -> EventPropagation {
        <Self as ParentElementImpl<_>>::on_mouse_scroll(self, pos, scroll)
    }

    fn on_keyboard_key(
        &mut self,
        key: Cx::Key,
        modifiers: &[Cx::Modifier],
        state: KeyState,
    ) -> EventPropagation {
        <Self as ParentElementImpl<_>>::on_keyboard_key(self, key, modifiers, state)
    }

    fn on_char_type(&mut self, c: char, modifiers: &[Cx::Modifier]) -> EventPropagation {
        <Self as ParentElementImpl<_>>::on_char_type(self, c, modifiers)
    }
}
