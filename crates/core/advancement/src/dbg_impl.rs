use crate::{Advancement, AdvancementCx, DisplayInfo};
use std::fmt::Debug;

impl<'r, Cx> Debug for DisplayInfo<'r, Cx>
where
    Cx: AdvancementCx + Debug,
    Cx::Content: Debug,
    Cx::Id: Debug,
    Cx::Settings<'r>: Debug,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisplayInfo")
            .field("title", &self.title)
            .field("description", &self.description)
            .field("icon", &self.icon)
            .field("background", &self.background)
            .field("frame", &self.frame)
            .field("show_toast", &self.show_toast)
            .field("announce_to_chat", &self.announce_to_chat)
            .field("hidden", &self.hidden)
            .field("pos", &self.pos)
            .finish()
    }
}

impl<'r, Cx> Debug for Advancement<'r, Cx>
where
    Cx: AdvancementCx + Debug,
    Cx::Content: Debug,
    Cx::Id: Debug,
    Cx::Settings<'r>: Debug,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Advancement")
            .field("parent", &self.parent)
            .field("display", &self.display)
            .finish()
    }
}
