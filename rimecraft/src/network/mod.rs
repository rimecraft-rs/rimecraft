/// Extentions for [`bytes::Buf`] with utility methods adapted to Minecraft's protocol.
/// It has deserialization of custom objects.
pub trait PacketBufExt {
    /// Reads an object from this buf as a compound NBT.
    fn decode_nbt<'a, T: serde::Deserialize<'a>>(&'a mut self) -> anyhow::Result<T>;
    fn read_string(&self, max_len: usize) -> anyhow::Result<String>;
}

impl<B: bytes::Buf> PacketBufExt for B {
    fn decode_nbt<'a, T: serde::Deserialize<'a>>(&'a mut self) -> anyhow::Result<T> {
        Ok(T::deserialize(&mut fastnbt::de::Deserializer::new(
            crate::nbt::BufInput(self),
            fastnbt::DeOpts::new(),
        ))?)
    }

    fn read_string(&self, max_len: usize) -> anyhow::Result<String> {
        let i = max_len * 3;
        let j = self.get_u16();
    }
}

/// Extentions for [`bytes::BufMut`] with utility methods adapted to Minecraft's protocol.
/// It has serialization of custom objects.
pub trait PacketBufMutExt {
    /// Writes an object to this buf as a compound NBT.
    fn encode_nbt<T: serde::Serialize>(&mut self, value: &T) -> anyhow::Result<()>;
}

impl<B: bytes::BufMut> PacketBufMutExt for B {
    fn encode_nbt<T: serde::Serialize>(&mut self, value: &T) -> anyhow::Result<()> {
        let mut vec = Vec::new();
        fastnbt::to_writer(&mut vec, value)?;
        self.put_slice(&vec);
        Ok(())
    }
}
