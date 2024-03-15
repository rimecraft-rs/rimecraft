use rimecraft_edcode::{Decode, Encode};
use rimecraft_global_cx::nbt_edcode::{ReadNbt, WriteNbt};
use rimecraft_registry::{ProvideRegistry, Reg};

use crate::{stack::ItemStackCx, ItemStack, RawItem};

impl<Cx> Encode for ItemStack<'_, Cx>
where
    Cx: ItemStackCx + for<'a> WriteNbt<&'a Cx::Compound>,
{
    fn encode<B>(&self, mut buf: B) -> Result<(), std::io::Error>
    where
        B: rimecraft_edcode::bytes::BufMut,
    {
        if self.count() == 0 {
            false.encode(buf)?;
        } else {
            true.encode(&mut buf)?;
            let item = self.item();
            item.encode(&mut buf)?;
            self.count().encode(&mut buf)?;
            if item.settings().max_damage.is_some() || item.settings().sync_nbt {
                if let Some(x) = self.nbt() {
                    Cx::write_nbt(x, buf.writer())?
                } else {
                    // @TODO: Write empty nbt tag.
                    todo!()
                }
            }
        }
        Ok(())
    }
}

impl<'r, Cx> Decode for ItemStack<'r, Cx>
where
    Cx: ItemStackCx + for<'a> ReadNbt<&'a Cx::Compound> + ProvideRegistry<'r, Cx::Id, RawItem<Cx>>,
{
    fn decode<B>(mut buf: B) -> Result<Self, std::io::Error>
    where
        B: rimecraft_edcode::bytes::Buf,
    {
        if bool::decode(&mut buf)? {
            let item = Reg::<'r, Cx::Id, RawItem<Cx>>::decode(&mut buf)?;
            let count = u8::decode(&mut buf)?;
            // @TODO: Handle null tags.
            let nbt = todo!();
            // Ok(ItemStack::with_nbt(item, count, nbt))
        } else {
            Ok(ItemStack::empty())
        }
    }
}
