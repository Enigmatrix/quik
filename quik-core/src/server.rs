use quik_util::*;

use crate::crypto::Crypto;
use crate::frame::Frame;
use crate::packet::Packet;
use crate::transport::Io;

// if this needs to be mutable inside, then it should use a mutex internally
pub trait Handler {
    fn handle<'a>(
        &self,
        packet: Packet<'a>,
        frames: impl Iterator<Item = Result<Frame<'a>>>,
    ) -> impl Future<Output = Result<()>>;
}

pub trait Server {
    type Crypto: Crypto;
    // UDP transport
    type Io: Io;
    type Handler: Handler;
}
