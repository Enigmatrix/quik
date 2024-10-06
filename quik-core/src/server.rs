use crate::crypto::Crypto;
use crate::frame::Frame;
use crate::packet::InitialPacket;
use crate::transport::Io;

use quik_util::*;

// if this needs to be mutable inside, then it should use a mutex internally
pub trait Handler {
    fn handle_initial_packet(&self, packet: InitialPacket) -> Result<()>;
    fn handle_frame(&self, frame: Frame) -> Result<()>;
}

pub trait Server {
    type Crypto: Crypto;
    // UDP transport
    type Io: Io;
    type Handler: Handler;
}
