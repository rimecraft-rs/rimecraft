//! Minecraft client UI framework.

use rimecraft_client_narration::Narratable;
use rimecraft_keyboard::ProvideKeyboardTy;
use rimecraft_mouse::{MousePos, MouseScroll, ProvideMouseTy};
use serde::{Deserialize, Serialize};

use crate::nav::WithNavIndex;

pub mod item;
pub mod nav;

/// The selection state of a UI component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

pub trait Element: WithNavIndex {
    fn mouse_moved(&mut self, pos: MousePos) {
        let _ = pos;
    }

    fn mouse_clicked<Cx>(&mut self, pos: MousePos, button: Cx::Button)
    where
        Cx: ProvideMouseTy,
    {
        drop((pos, button));
    }

    fn mouse_released<Cx>(&mut self, pos: MousePos, button: Cx::Button)
    where
        Cx: ProvideMouseTy,
    {
        drop((pos, button));
    }

    fn mouse_dragged<Cx>(&mut self, pos: MousePos, delta_pos: MousePos, button: Cx::Button)
    where
        Cx: ProvideMouseTy,
    {
        drop((pos, delta_pos, button));
    }

    fn mouse_scrolled(&mut self, pos: MousePos, scroll: MouseScroll) {
        let _ = (pos, scroll);
    }

    fn key_pressed<Cx>(&mut self, key: Cx::Key, modifiers: &[Cx::Modifier])
    where
        Cx: ProvideKeyboardTy,
    {
        drop((key, modifiers));
    }
}
