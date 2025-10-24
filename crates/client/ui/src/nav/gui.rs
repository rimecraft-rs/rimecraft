//! Navigate through the GUI hierarchy.

use std::fmt::Debug;

use rimecraft_arena::Arena;
use rimecraft_cell::Cell;
use rimecraft_local_cx::{LocalContext, LocalContextExt, WithLocalCx};

use crate::{
    Focusable, ProvideUiTy,
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

#[derive(Clone)]
#[allow(clippy::exhaustive_enums)]
pub enum GuiNavigationPath<'g, Cx>
where
    Cx: ProvideUiTy<'g>,
{
    Node {
        element: <Cx::Arena as Arena>::Handle,
        child: Box<GuiNavigationPath<'g, Cx>>,
    },
    Leaf(<Cx::Arena as Arena>::Handle),
}

impl<'g, Cx> Debug for GuiNavigationPath<'g, Cx>
where
    Cx: ProvideUiTy<'g>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiNavigationPath::Node { .. } => f.debug_struct("GuiNavigationPath::Node").finish(),
            GuiNavigationPath::Leaf(_) => f.debug_struct("GuiNavigationPath::Leaf").finish(),
        }
    }
}

pub type GuiNavigationPathWithCx<'g, Cx> =
    WithLocalCx<&'g GuiNavigationPath<'g, Cx>, <Cx as ProvideUiTy<'g>>::ArenaLocalContext>;

impl<'g, Cx> Focusable<Cx> for GuiNavigationPathWithCx<'g, Cx>
where
    Cx: ProvideUiTy<'g>,
{
    fn is_focused(&self) -> bool {
        match &self.inner {
            GuiNavigationPath::Node { element, child: _ } => {
                if let Some(cell) = self.local_cx.acquire().get(*element) {
                    cell.read().is_focused()
                } else {
                    false
                }
            }
            GuiNavigationPath::Leaf(element) => self
                .local_cx
                .acquire()
                .get(*element)
                .map_or(false, |cell| cell.read().is_focused()),
        }
    }

    fn set_focused(&mut self, focused: bool) {
        match &self.inner {
            GuiNavigationPath::Node { element, child } => {
                if let Some(cell) = self.local_cx.acquire().get(*element) {
                    {
                        cell.write().set_focused(false);
                    }

                    self.local_cx.with(child.as_ref()).set_focused(focused);
                }
            }
            GuiNavigationPath::Leaf(element) => {
                if let Some(cell) = self.local_cx.acquire().get(*element) {
                    cell.write().set_focused(focused);
                }
            }
        }
    }
}

impl<'g, Cx> GuiNavigationPath<'g, Cx>
where
    Cx: ProvideUiTy<'g>,
{
    pub fn handle(&self) -> <Cx::Arena as Arena>::Handle {
        match self {
            GuiNavigationPath::Node { element, .. } => *element,
            GuiNavigationPath::Leaf(element) => *element,
        }
    }

    pub fn leaf(element: <Cx::Arena as Arena>::Handle) -> Self
    where
        Cx: ProvideUiTy<'g>,
    {
        GuiNavigationPath::Leaf(element)
    }

    pub fn node(element: <Cx::Arena as Arena>::Handle, child: GuiNavigationPath<'g, Cx>) -> Self
    where
        Cx: ProvideUiTy<'g>,
    {
        GuiNavigationPath::Node {
            element,
            child: Box::new(child),
        }
    }

    pub fn path(
        leaf: <Cx::Arena as Arena>::Handle,
        parents: Vec<<Cx::Arena as Arena>::Handle>,
    ) -> Self
    where
        Cx: ProvideUiTy<'g>,
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
