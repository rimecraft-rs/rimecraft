//! Navigate through the GUI hierarchy.

use std::fmt::Debug;

use crate::{
    ContainerElement, Element, ProvideUiTy,
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
            NavDirection::Down // tab navigates down
        } else {
            NavDirection::Up // shift+tab navigates up
        }
    }
}

/// A path to traverse down an UI hierarchy through navigation.
#[allow(clippy::exhaustive_enums)]
pub enum GuiNavigationPath<'a, Cx>
where
    Cx: ProvideUiTy,
{
    /// A leaf node that contains an [`Element`] with no child paths.
    Leaf(&'a dyn Element<Cx>),
    /// An intermediate node that contains a [`ContainerElement`] with a child path.
    Node(&'a dyn ContainerElement<Cx>, Box<GuiNavigationPath<'a, Cx>>),
}

impl<Cx> Debug for GuiNavigationPath<'_, Cx>
where
    Cx: ProvideUiTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(element) => {
                let type_name = std::any::type_name_of_val(&**element);
                write!(
                    f,
                    "Leaf({} @ {:p})",
                    type_name,
                    std::ptr::from_ref(*element)
                )
            }
            Self::Node(container, child) => {
                let type_name = std::any::type_name_of_val(&**container);
                write!(
                    f,
                    "Node({} @ {:p}, child={:?})",
                    type_name,
                    std::ptr::from_ref(*container),
                    child
                )
            }
        }
    }
}

impl<'a, Cx> GuiNavigationPath<'a, Cx>
where
    Cx: ProvideUiTy,
{
    /// Creates a new [`GuiNavigationPath`] from a leaf element.
    pub fn new_leaf(leaf: &'a dyn Element<Cx>) -> Self {
        Self::Leaf(leaf)
    }

    /// Creates a new [`GuiNavigationPath`] from a container element and a child path.
    pub fn new_node(container: &'a dyn ContainerElement<Cx>, child_path: Self) -> Self {
        Self::Node(container, Box::new(child_path))
    }

    /// Creates a new [`GuiNavigationPath`] from a leaf element and its parent elements.
    pub fn new<I>(leaf: &'a dyn Element<Cx>, parents: I) -> Self
    where
        I: IntoIterator<Item = &'a dyn ContainerElement<Cx>>,
        I::IntoIter: DoubleEndedIterator,
    {
        parents
            .into_iter()
            .rev()
            .fold(Self::Leaf(leaf), |acc, parent| Self::new_node(parent, acc))
    }
}

impl<Cx> GuiNavigationPath<'_, Cx>
where
    Cx: ProvideUiTy,
{
    /// The containing [`Element`] of this path.
    pub fn element(&self) -> &dyn Element<Cx> {
        match self {
            Self::Leaf(element) => *element,
            Self::Node(parent, _) => *parent,
        }
    }

    /// Sets the focus state of this path.
    pub fn set_focused(&self, focused: bool) {
        match self {
            Self::Leaf(element) => element.set_focused(focused),
            Self::Node(parent, child_path) => {
                if focused {
                    parent.set_focused_child(Some(child_path.element()));
                } else {
                    parent.set_focused_child(None);
                }
                child_path.set_focused(focused);
            }
        }
    }
}
