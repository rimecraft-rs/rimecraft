pub mod listener;
pub mod packet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

impl State {
    pub fn id(&self) -> &str {
        match self {
            State::Handshaking => "handshake",
            State::Play => "play",
            State::Status => "status",
            State::Login => "login",
            State::Configuration => "configuration",
        }
    }
}
