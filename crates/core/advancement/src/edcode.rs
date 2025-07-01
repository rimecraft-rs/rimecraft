use rimecraft_edcode2::{Buf, BufMut, Decode, Encode};
use rimecraft_global_cx::{
    edcode::Nbt,
    nbt::{ReadNbt, WriteNbt},
};
use rimecraft_item::{component::RawErasedComponentType, ItemStack, RawItem};
use rimecraft_local_cx::{dyn_cx::AsDynamicContext, LocalContext, WithLocalCx};
use rimecraft_registry::{ProvideRegistry, Registry};
use rimecraft_text::Text;

use crate::{AdvancementCx, DisplayInfo, Frame};

/// Additional requirements when enabling `edcode` on [`Advancement`].
pub trait AdvancementEdcodeCx: AdvancementCx {
    /// Given [`FrameData::name`], returns corresponding [`Frame`].
    fn frame_fmt(name: &str) -> Frame;
}

impl<'r, Cx, B, L> Encode<WithLocalCx<B, L>> for DisplayInfo<'r, Cx>
where
    Cx: AdvancementEdcodeCx
        + LocalContext<Registry<Cx::Id, RawItem<'r, Cx>>>
        + for<'a, 'b> WriteNbt<&'a &'b Text<Cx>>,
    B: BufMut,
    L: AsDynamicContext,
    Cx::Id: for<'a> Encode<&'a mut B>,
{
    fn encode(
        &self,
        mut buf: WithLocalCx<B, L>,
    ) -> Result<(), rimecraft_edcode2::BoxedError<'static>> {
        Nbt::<&Text<Cx>, Cx>::new(&self.title).encode(&mut buf)?;
        Nbt::<&Text<Cx>, Cx>::new(&self.description).encode(&mut buf)?;
        self.icon.encode(buf.as_mut())?;
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
            .map_or(Ok(()), |bg| bg.encode(buf.as_mut().inner))?;
        self.pos.0.encode(&mut buf)?;
        self.pos.1.encode(&mut buf)?;
        Ok(())
    }
}

impl<'de, 'r, Cx, B> Decode<'de, B> for DisplayInfo<'r, Cx>
where
    Cx: AdvancementEdcodeCx
        + ProvideRegistry<'r, Cx::Id, RawItem<'r, Cx>>
        + ProvideRegistry<'r, Cx::Id, RawErasedComponentType<'r, Cx>>
        + ReadNbt<Text<Cx>>,
    B: Buf,
    Cx::Id: for<'a, 'b> Decode<'de, &'a mut &'b mut B> + for<'a> Decode<'de, &'a mut B>,
{
    #[allow(unused_variables)]
    fn decode(mut buf: B) -> Result<Self, rimecraft_edcode2::BoxedError<'de>> {
        let title = Nbt::<Text<Cx>, Cx>::decode(&mut buf)?.into_inner();
        let description = Nbt::<Text<Cx>, Cx>::decode(&mut buf)?.into_inner();
        let icon: ItemStack<'r, Cx> = Decode::decode(&mut buf)?;
        let frame: Frame = Decode::decode(&mut buf)?;
        let i: i32 = Decode::decode(&mut buf)?;
        let background: Option<Cx::Id> = ((i & 1) != 0)
            .then(|| Decode::decode(&mut buf))
            .transpose()?;
        let show_toast = (i & 2) != 0;
        let hidden = (i & 4) != 0;
        let x: f32 = Decode::decode(&mut buf)?;
        let y: f32 = Decode::decode(&mut buf)?;
        Ok(Self {
            title,
            description,
            icon,
            background,
            frame,
            show_toast,
            announce_to_chat: false,
            hidden,
            pos: (x, y),
        })
    }
}
