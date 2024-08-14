use crate::{AdvancementCx, DisplayInfo};
use std::fmt::Debug;

impl<Cx> Debug for DisplayInfo<'_, Cx>
where
    Cx: AdvancementCx + Debug,
    Cx::Content: Debug,
    Cx::Id: Debug,
    Cx::Compound: Debug,
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
