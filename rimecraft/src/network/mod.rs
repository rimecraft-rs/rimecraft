/// Extentions for [`bytes::Buf`] with utility methods adapted to Minecraft's protocol.
/// It has serialization and deserialization of custom objects.
pub trait PacketBufExt: bytes::Buf {
    /// Reads an object from this buf as a compound NBT.
    fn decode<'a, T: serde::Deserialize<'a>>(&'a mut self) -> Result<T, fastnbt::error::Error> {
        fastnbt::from_bytes(self.chunk())
    }
}

/// Extentions for [`bytes::BufMut`] and [`bytes::Buf`] with utility methods adapted to Minecraft's protocol.
/// It has serialization and deserialization of custom objects.
pub trait PacketBufMutExt: PacketBufExt + bytes::BufMut {}
