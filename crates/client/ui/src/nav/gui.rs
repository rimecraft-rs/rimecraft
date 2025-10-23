//! Navigate through the GUI hierarchy.

use std::fmt::Debug;

use rimecraft_keyboard::ProvideKeyboardTy;
use rimecraft_mouse::ProvideMouseTy;

use crate::{
    Element, ImmutablyFocusable, ParentElement, ProvideUiTy,
    nav::{NavAxis, NavDirection},
};

/// A navigation action in the GUI.
pub trait GuiNavigation {
    /// The direction of the navigation.
    fn direction(&self) -> NavDirection;
}

/// A navigation action that always goes down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuiDownNavigation;

impl GuiNavigation for GuiDownNavigation {
    fn direction(&self) -> NavDirection {
        NavDirection::Down // Always navigates down
    }
}

/// Navigation triggered by arrow keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuiArrowNavigation(pub NavDirection);

impl GuiNavigation for GuiArrowNavigation {
    fn direction(&self) -> NavDirection {
        match self.0.axis() {
            NavAxis::Vertical => self.0,
            NavAxis::Horizontal => NavDirection::Down, // Horizontal arrow directions will lead to a deeper navigation level
        }
    }
}

/// Navigation triggered by Tab key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuiTabNavigation {
    /// Whether the navigation is forward (Tab) or backward (Shift+Tab).
    pub forward: bool,
}

impl GuiNavigation for GuiTabNavigation {
    fn direction(&self) -> NavDirection {
        if self.forward {
            NavDirection::Down // Tab navigates down
        } else {
            NavDirection::Up // Shift+Tab navigates up
        }
    }
}

#[allow(clippy::exhaustive_enums)]
pub enum GuiNavigationPath<'g, Cx>
where
    Cx: ProvideUiTy,
{
    Node {
        element: &'g dyn ParentElement<Cx>,
        child: Box<GuiNavigationPath<'g, Cx>>,
    },
    Leaf(&'g dyn Element<Cx>),
}

impl<Cx> Debug for GuiNavigationPath<'_, Cx>
where
    Cx: ProvideUiTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiNavigationPath::Node { .. } => f.debug_struct("GuiNavigationPath::Node").finish(),
            GuiNavigationPath::Leaf(_) => f.debug_struct("GuiNavigationPath::Leaf").finish(),
        }
    }
}

impl<Cx> ImmutablyFocusable<Cx> for GuiNavigationPath<'_, Cx>
where
    Cx: ProvideUiTy,
{
    fn is_focused(&self) -> bool {
        match self {
            GuiNavigationPath::Node { element, child } => {
                if let Some(index) = element.child_index(child.element()) {
                    element.children()[index].is_focused()
                } else {
                    false
                }
            }
            GuiNavigationPath::Leaf(element) => element.is_focused(),
        }
    }

    fn set_focused(&self, focused: bool) {
        match self {
            GuiNavigationPath::Node { element, child } => {
                element.set_focused(false);

                if let Some(index) = element.child_index(child.element()) {
                    element.children()[index].set_focused(focused);
                }

                child.set_focused(focused);
            }
            GuiNavigationPath::Leaf(element) => {
                element.set_focused(focused);
            }
        }
    }
}

impl<'g, Cx> GuiNavigationPath<'g, Cx>
where
    Cx: ProvideUiTy,
{
    pub fn element(&self) -> &dyn Element<Cx> {
        match self {
            GuiNavigationPath::Node { element, .. } => *element,
            GuiNavigationPath::Leaf(element) => *element,
        }
    }

    pub fn leaf<E>(element: &'g E) -> Self
    where
        E: Element<Cx> + 'static,
        Cx: ProvideKeyboardTy + ProvideMouseTy,
    {
        GuiNavigationPath::Leaf(element)
    }

    pub fn node<E>(element: &'g E, child: GuiNavigationPath<'g, Cx>) -> Self
    where
        E: ParentElement<Cx> + 'static,
        Cx: ProvideKeyboardTy + ProvideMouseTy,
    {
        GuiNavigationPath::Node {
            element,
            child: Box::new(child),
        }
    }

    pub fn path<E>(leaf: &'g E, parents: Vec<&'g dyn ParentElement<Cx>>) -> Self
    where
        E: Element<Cx> + 'static,
        Cx: ProvideKeyboardTy + ProvideMouseTy,
    {
        let mut current = GuiNavigationPath::leaf(leaf);

        for parent in parents.into_iter().rev() {
            current = GuiNavigationPath::Node {
                element: parent,
                child: Box::new(current),
            };
        }

        current
    }
}
