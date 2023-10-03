pub mod listener;
pub mod packet;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Handshaking = -1,
    Play = 0,
    Status = 1,
    Login = 2,
}

impl State {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            -1 => Some(Self::Handshaking),
            0 => Some(Self::Play),
            1 => Some(Self::Status),
            2 => Some(Self::Login),
            _ => None,
        }
    }
}
