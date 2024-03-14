use rimecraft_edcode::Encode;

use crate::{stack::ItemStackCx, ItemStack};

impl<Cx> Encode for ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
{
    fn encode<B>(&self, buf: B) -> Result<(), std::io::Error>
    where
        B: rimecraft_edcode::bytes::BufMut,
    {
        if self.count() == 0 {
            false.encode(buf)?;
        } else {
            true.encode(buf)?;
        }
        Ok(())
    }
}
