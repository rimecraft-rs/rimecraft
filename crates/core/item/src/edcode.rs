use component::{RawErasedComponentType, changes::ComponentChanges, map::ComponentMap};
use edcode2::{Buf, BufExt, BufMut, BufMutExt, Decode, Encode};
use local_cx::{LocalContext, WithLocalCx, dyn_cx::AsDynamicContext};
use rimecraft_registry::{Reg, Registry};

use crate::{Item, ItemSettings, ItemStack, RawItem, stack::ItemStackCx};

impl<Cx, B, L> Encode<WithLocalCx<B, L>> for ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
    B: BufMut,
    L: AsDynamicContext,
{
    fn encode(&self, mut buf: WithLocalCx<B, L>) -> Result<(), edcode2::BoxedError<'static>> {
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

impl<'r, 'de, Cx, B, L> Decode<'de, WithLocalCx<B, L>> for ItemStack<'r, Cx>
where
    Cx: ItemStackCx<Id: for<'b> Decode<'de, WithLocalCx<&'b mut B, L>>>,
    B: Buf,
    L: LocalContext<&'r Registry<Cx::Id, RawItem<'r, Cx>>>
        + LocalContext<&'r Registry<Cx::Id, RawErasedComponentType<'r, Cx>>>
        + AsDynamicContext,
{
    fn decode(mut buf: WithLocalCx<B, L>) -> Result<Self, edcode2::BoxedError<'de>> {
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
