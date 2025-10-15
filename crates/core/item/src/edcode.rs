use component::{RawErasedComponentType, changes::ComponentChanges, map::ComponentMap};
use edcode2::{Buf, BufExt, BufMut, BufMutExt, Decode, Encode};
use local_cx::{ForwardToWithLocalCx, LocalContext, WithLocalCx};
use rimecraft_registry::{Reg, Registry};

use crate::{Item, ItemSettings, ItemStack, RawItem, stack::ItemStackCx};

impl<'a, Cx, Fw> Encode<Fw> for ItemStack<'a, Cx>
where
    Cx: ItemStackCx,
    Fw: ForwardToWithLocalCx<Forwarded: BufMut, LocalCx = Cx::LocalContext<'a>>,
{
    fn encode(&self, buf: Fw) -> Result<(), edcode2::BoxedError<'static>> {
        let mut buf = buf.forward();
        if self.is_empty() {
            buf.put_variable(0u32);
            Ok(())
        } else {
            buf.put_variable(self.count());
            let item = self.item();
            item.encode(&mut buf)?;
            self.count().encode(&mut buf)?;
            self.components()
                .changes()
                .ok_or("components not patched")?
                .encode(buf)
        }
    }
}

impl<'r, 'de, Cx, Fw> Decode<'de, Fw> for ItemStack<'r, Cx>
where
    Cx: ItemStackCx<Id: for<'b> Decode<'de, WithLocalCx<&'b mut Fw::Forwarded, Fw::LocalCx>>>,
    Fw: ForwardToWithLocalCx<Forwarded: Buf, LocalCx = Cx::LocalContext<'r>>,
    Cx::LocalContext<'r>: LocalContext<&'r Registry<Cx::Id, RawItem<'r, Cx>>>
        + LocalContext<&'r Registry<Cx::Id, RawErasedComponentType<'r, Cx>>>,
{
    fn decode(buf: Fw) -> Result<Self, edcode2::BoxedError<'de>> {
        let mut buf = buf.forward();
        let count: u32 = buf.get_variable();
        if count == 0 {
            Ok(ItemStack::empty(buf.local_cx))
        } else {
            let item = Item::<'r, Cx>::decode(buf.as_mut())?;
            let changes = ComponentChanges::<'r, 'r, Cx>::decode(buf)?;
            Ok(ItemStack::with_component(
                item,
                count,
                ComponentMap::with_changes(Reg::to_value(item).settings().components(), changes),
            ))
        }
    }
}
