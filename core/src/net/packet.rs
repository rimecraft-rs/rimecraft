pub trait Packet: super::Encode {
    type Listener:PacketListener;

    fn apply(&self, listener: Self::Listener) -> anyhow::Result<()>;
    fn is_writing_err_skippable(&self) -> bool {
        true
    }
}

pub trait PacketListener {
    const SHOULD_CRASH_ON_EXCEPTION: bool = true;

    ///TODO: Need to implement net.minecraft.Text
    fn on_disconncted(&self, reason: ()) -> anyhow::Result<()>;
    fn is_conn_open(&self) -> bool;
}
