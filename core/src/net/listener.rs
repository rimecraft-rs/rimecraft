use crate::text::Text;

pub trait Listener {
    const SHOULD_CRASH_ON_EXCEPTION: bool;

    fn disconncted<T>(&mut self, reason: T) -> anyhow::Result<()>
    where
        T: Text;

    fn is_conn_open(&self) -> bool;
}

/// Represent [`Listener`] types that are able to accept
/// a certain type of packet when applying it.
pub trait Accept<T: ?Sized>: Listener
where
    T: super::packet::Packet<Self>,
{
    /// Accept the packet.
    fn accept_packet(&mut self, packet: &T) -> anyhow::Result<()>;
}

impl<T> Accept<()> for T
where
    T: Listener,
{
    #[inline]
    fn accept_packet(&mut self, _packet: &()) -> anyhow::Result<()> {
        Ok(())
    }
}

pub trait Tick: Listener {
    fn tick(&mut self);
}
