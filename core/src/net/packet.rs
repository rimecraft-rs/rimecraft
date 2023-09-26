pub trait Packet {
    fn write<B>(&self, buf: &mut B)->anyhow::Result<()>
    where
        B:bytes::BufMut;
}