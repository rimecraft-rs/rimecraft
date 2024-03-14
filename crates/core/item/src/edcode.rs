use rimecraft_edcode::Encode;

use crate::{stack::ItemStackCx, ItemStack};

impl<Cx> Encode for ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
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
            self.count().encode(buf)?;
            if item.settings().max_damage.is_some() || item.settings().sync_nbt {
                todo!()
            }
        }
        Ok(())
    }
}
