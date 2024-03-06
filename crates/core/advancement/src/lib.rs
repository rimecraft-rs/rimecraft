use rimecraft_identifier::{
    vanilla::{Namespace, Path},
    Identifier,
};
use rimecraft_item::ItemStack;
use rimecraft_text::{Text, Texts};

pub struct Advancement<T, Id>
where
    T: Texts,
{
    pub parent: Option<Id>,
    pub display: Option<DisplayInfo<T, Id>>,
}

/// # MCJE Reference
///
/// `net.minecraft.advancement.AdvancementDisplay` in yarn.
pub struct DisplayInfo<'r, T, Id, Cx>
where
    T: Texts,
{
    title: T,
    description: T,
    /// @TODO: ItemStack
    icon: ItemStack<'r, Id, Cx>,
    background: Option<Id>,
    frame: Frame<(), ()>,
    show_toast: bool,
    announce_to_chat: bool,
    hidden: bool,
    pos: (f32, f32),
}

impl<'r,T, Id,Cx> DisplayInfo<'r,T, Id,Cx>
where
    T: Texts,
{
    pub fn new(
        title: T,
        description: T,
        icon: ItemStack,
        background: Option<Id>,
        frame: Frame<(), ()>,
        show_toast: bool,
        announce_to_chat: bool,
        hidden: bool,
    ) -> Self {
        Self {
            title,
            description,
            icon,
            background,
            frame,
            show_toast,
            announce_to_chat,
            hidden,
            pos: (0., 0.),
        }
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.pos = (x, y);
    }

    pub fn title(&self) -> &T {
        &self.title
    }
}

pub struct Frame<I, F> {
    id: I,
    format: F,
}
