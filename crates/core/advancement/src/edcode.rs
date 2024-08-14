use rimecraft_edcode2::{Buf, BufMut, Decode, Encode};
use rimecraft_global_cx::nbt_edcode::{ReadNbt, WriteNbt};
use rimecraft_item::{ItemStack, RawItem};
use rimecraft_registry::ProvideRegistry;

use crate::{AdvancementCx, DisplayInfo, Frame};

/// Additional requirements when enabling `edcode` on [`Advancement`].
pub trait AdvancementEdcodeCx:
    AdvancementCx
    + for<'a> WriteNbt<Option<&'a Self::Compound>>
    + ReadNbt<Option<Self::Compound>>
    + for<'r> ProvideRegistry<'r, Self::Id, RawItem<Self>>
{
    /// Given [`FrameData::name`], returns corresponding [`Frame`].
    fn frame_fmt(name: &str) -> Frame;
}

impl<Cx, B> Encode<B> for DisplayInfo<'_, Cx>
where
    Cx: AdvancementEdcodeCx,
    B: BufMut,
    Cx::Id: for<'a> Encode<&'a mut B>,
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
        i.encode(&mut buf)?;
        self.background
            .as_ref()
            .map_or(Ok(()), |bg| bg.encode(&mut buf))?;
        self.pos.0.encode(&mut buf)?;
        self.pos.1.encode(&mut buf)?;
        Ok(())
    }
}

impl<'de, 'r, Cx, B> Decode<'de, B> for DisplayInfo<'r, Cx>
where
    Cx: AdvancementEdcodeCx,
    B: Buf,
    Cx::Id: for<'a> Decode<'de, &'a mut B>,
{
    #[allow(unused_variables)]
    fn decode(mut buf: B) -> Result<Self, rimecraft_edcode2::BoxedError<'de>> {
        // TODO: Decode `title` and `description`.
        let stack: ItemStack<'r, Cx> = Decode::decode(&mut buf)?;
        let frame: Frame = Decode::decode(&mut buf)?;
        let i: i32 = Decode::decode(&mut buf)?;
        let background: Option<Cx::Id> = ((i & 1) != 0)
            .then(|| Decode::decode(&mut buf))
            .transpose()?;
        let show_toast = (i & 2) != 0;
        let hidden = (i & 4) != 0;
        let x: f32 = Decode::decode(&mut buf)?;
        let y: f32 = Decode::decode(&mut buf)?;
        // TODO: Construct the object with [`Self::new`].
        todo!()
    }
}
