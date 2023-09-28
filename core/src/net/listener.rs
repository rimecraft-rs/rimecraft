pub trait Listener {
    const SHOULD_CRASH_ON_EXCEPTION: bool;

    //TODO: Need to implement net.minecraft.Text
    fn disconncted(&mut self, reason: ()) -> anyhow::Result<()>;
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
