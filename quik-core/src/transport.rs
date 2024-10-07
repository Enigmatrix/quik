use std::iter::Empty;

use crate::crypto::Crypto;
use crate::frame::Frame;
use crate::packet::{Packet, RemainingBuf};
use crate::server::Handler;
use quik_util::*;

pub trait Io {
    fn send(&self, data: &[u8]) -> impl Future<Output = Result<()>>;
    fn recv(&self, data: &mut [u8]) -> impl Future<Output = Result<()>>;
    fn close(self) -> impl Future<Output = ()>;
}

pub struct Connection<C: Crypto, I: Io, H: Handler> {
    crypto: C,
    io: I,
    handler: H,
}

impl<C: Crypto, I: Io, H: Handler> Connection<C, I, H> {
    pub fn send(&self, data: &[u8]) {
        // Send data
    }

    pub async fn recv(&self, data: &[u8]) -> Result<()> {
        let (packet, remainder) = Packet::parse(&self.crypto, data).await?;
        match remainder {
            RemainingBuf::Decrypted(data) => {
                self.handler
                    .handle(packet, Frame::parse_multiple(&data))
                    .await?;
            }
            RemainingBuf::None => {
                self.handler.handle(packet, Empty::default()).await?;
            }
        }
        Ok(())
    }

    pub fn close(self) {
        // Close connection
    }
}
