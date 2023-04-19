pub mod packet;

use std::net::SocketAddr;

#[derive(PartialEq)]
pub enum Proxy {
    NoProxy,
    SOCKS(SocketAddr, Option<ProxyPassword>),
}

#[derive(PartialEq)]
pub struct ProxyPassword(pub String, pub String);
