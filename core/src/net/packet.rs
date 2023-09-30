pub mod c2s;
pub mod s2c;

use anyhow::Ok;

use super::{listener::*, Encode};

const QUERY_MAX_PAYLOAD_LEN: usize = 1048576;

pub trait Packet<L: ?Sized>: super::BytesEncode
where
    L: super::listener::Accept<Self>,
{
    #[inline]
    fn apply(&mut self, listener: &mut L) -> anyhow::Result<()> {
        listener.accept_packet(self)?;
        self.post_apply()
    }

    #[inline]
    fn post_apply(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Returns whether a throwable in writing of this packet
    /// allows the connection to simply skip the packet's sending
    /// than disconnecting.
    #[inline]
    fn is_writing_err_skippable(&self) -> bool {
        false
    }
}

/// Provides an abstraction to [`Packet::apply`], without complex
/// type restrictions.
///
/// Used in [`Bundled`].
pub unsafe trait AbstPacketApply<T>
where
    T: ?Sized + Listener,
{
    /// Apply this packet to a listener.
    fn apply(&mut self, listener: &mut T) -> anyhow::Result<()>;
}

unsafe impl<T, L> AbstPacketApply<L> for T
where
    T: Packet<L> + ?Sized,
    L: ?Sized + Accept<T>,
{
    #[inline]
    fn apply(&mut self, listener: &mut L) -> anyhow::Result<()> {
        Packet::apply(self, listener)
    }
}

pub struct Bundled<T>
where
    T: ?Sized + Listener,
{
    packets: Vec<Box<dyn AbstPacketApply<T>>>,
}

impl<T> Bundled<T>
where
    T: Listener + ?Sized,
{
    #[inline]
    pub fn new(packets: Vec<Box<dyn AbstPacketApply<T>>>) -> Self {
        Self { packets }
    }

    #[inline]
    pub fn packets(&self) -> &[Box<dyn AbstPacketApply<T>>] {
        &(self.packets)
    }
}

impl<T> Encode for Bundled<T>
where
    T: Listener + ?Sized,
{
    #[inline]
    fn encode<B>(&self, _buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        Ok(())
    }
}

impl<T: Listener> Packet<T> for () {
    #[inline]
    fn post_apply(&mut self) -> anyhow::Result<()> {
        unreachable!("should be handled by pipelines")
    }
}