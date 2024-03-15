use rimecraft_edcode::Encode;
use rimecraft_global_cx::nbt_edcode::WriteNbt;

use crate::{stack::ItemStackCx, ItemStack};

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
                    // Write empty nbt tag.
                    todo!()
                }
            }
        }
        Ok(())
    }
}
