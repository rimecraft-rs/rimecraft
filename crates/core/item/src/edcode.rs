use component::{changes::ComponentChanges, map::ComponentMap, RawErasedComponentType};
use edcode2::{Buf, BufExt, BufMut, BufMutExt, Decode, Encode};
use rimecraft_registry::{ProvideRegistry, Reg};

use crate::{stack::ItemStackCx, Item, ItemSettings, ItemStack, RawItem};

impl<'r, Cx, B> Encode<B> for ItemStack<'r, Cx>
where
    Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<'r, Cx>>,
    B: BufMut,
{
    fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
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

impl<'r, 'de, Cx, B> Decode<'de, B> for ItemStack<'r, Cx>
where
    'r: 'de,
    Cx: ItemStackCx<Id: for<'b> Decode<'de, &'b mut B>>
        + ProvideRegistry<'r, Cx::Id, RawItem<'r, Cx>>
        + ProvideRegistry<'r, Cx::Id, RawErasedComponentType<'r, Cx>>,
    B: Buf,
{
    fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
        let count: u32 = buf.get_variable();
        if count == 0 {
            Ok(ItemStack::empty())
        } else {
            let item = Item::<'r, Cx>::decode(&mut buf)?;
            let changes = ComponentChanges::<'r, 'r, Cx>::decode(buf)?;
            Ok(ItemStack::with_component(
                item,
                count,
                ComponentMap::with_changes(Reg::into_inner(item).settings().components(), changes),
            ))
        }
    }
}
