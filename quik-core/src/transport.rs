use std::iter::Empty;

use quik_util::*;

use crate::crypto::Crypto;
use crate::handler::Handler;
use crate::wire::packet::RemainingBuf;
use crate::wire::{Frame, Packet};

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
  pub fn send<'a>(&self, packet: Packet, frames: impl Iterator<Item = Frame<'a>>) {
    // Send data using the underlying UDP transport
  }

  pub async fn recv(&self, data: &[u8]) -> Result<()> {
    let (packet, remainder) = Packet::parse(&self.crypto, data).await?;
    match remainder {
      RemainingBuf::Decrypted(data) => {
        self
          .handler
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
