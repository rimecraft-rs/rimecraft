use rimecraft_edcode::{decode_cow_str, Decode, Encode};

use crate::{AdvancementEdcodeCx, Frame};

impl<Cx> Encode for Frame<Cx>
where
    Cx: AdvancementEdcodeCx,
{
    fn encode<B>(&self, buf: B) -> Result<(), std::io::Error>
    where
        B: rimecraft_edcode::bytes::BufMut,
    {
        self.data.name.encode(buf)?;
        Ok(())
    }
}

impl<Cx> Decode for Frame<Cx>
where
    Cx: AdvancementEdcodeCx,
{
    fn decode<B>(mut buf: B) -> Result<Self, std::io::Error>
    where
        B: rimecraft_edcode::bytes::Buf,
    {
        let name = decode_cow_str(&mut buf)?;
        Ok(Cx::frame_fmt(&name))
    }
}
