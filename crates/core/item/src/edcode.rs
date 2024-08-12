use edcode2::{Buf, BufExt, BufMut, BufMutExt, Decode, Encode};
use rimecraft_global_cx::nbt_edcode::{ReadNbt, WriteNbt};
use rimecraft_registry::{ProvideRegistry, Reg};

use crate::{stack::ItemStackCx, ItemStack, RawItem};

impl<Cx, B> Encode<B> for ItemStack<'_, Cx>
where
    Cx: ItemStackCx + for<'a> WriteNbt<Option<&'a Cx::Compound>>,
    B: BufMut,
{
    fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
        if self.count() == 0 {
            buf.put_bool(false);
        } else {
            buf.put_bool(true);
            let item = self.item();
            item.encode(&mut buf)?;
            self.count().encode(&mut buf)?;
            if item.settings().max_damage.is_some() || item.settings().sync_nbt {
                Cx::write_nbt(self.nbt(), buf.writer())?
            }
        }
        Ok(())
    }
}

impl<'r, 'de, Cx, B> Decode<'de, B> for ItemStack<'r, Cx>
where
    Cx: ItemStackCx + ReadNbt<Option<Cx::Compound>> + ProvideRegistry<'r, Cx::Id, RawItem<Cx>>,
    B: Buf,
{
    fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
        if buf.get_bool() {
            let item = Reg::<'r, Cx::Id, RawItem<Cx>>::decode(&mut buf)?;
            let count = buf.get_u8();
            let nbt = Cx::read_nbt(buf.reader())?;
            Ok(ItemStack::with_nbt(item, count, nbt))
        } else {
            Ok(ItemStack::empty())
        }
    }
}
