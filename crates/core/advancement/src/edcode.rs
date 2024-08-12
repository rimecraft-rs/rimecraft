use rimecraft_edcode2::{BufMut, Decode, Encode};
use rimecraft_global_cx::nbt_edcode::WriteNbt;

use crate::{AdvancementCx, DisplayInfo, Frame};

/// Additional requirements when enabling `edcode` on [`Advancement`].
pub trait AdvancementEdcodeCx:
    AdvancementCx + for<'a> WriteNbt<Option<&'a Self::Compound>>
{
    /// Given [`FrameData::name`], returns corresponding [`Frame`].
    fn frame_fmt(name: &str) -> Frame;
}

impl<Cx, B> Encode<B> for DisplayInfo<'_, Cx>
where
    Cx: AdvancementEdcodeCx,
    B: BufMut,
{
    fn encode(&self, mut buf: B) -> Result<(), rimecraft_edcode2::BoxedError<'static>> {
        // TODO: `RawText` doesn't implement edcode.
        // TODO: Encode `title` and `description`.
        self.icon.encode(&mut buf)?;
        self.frame.encode(&mut buf)?;
        let mut i = 0_i32;
        if self.background.is_some() {
            i |= 1;
        }
        if self.show_toast {
            i |= 2;
        }
        if self.hidden {
            i |= 4;
        }
        i.encode(buf)?;
        todo!()
    }
}
