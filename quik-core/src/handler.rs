use quik_util::*;

use crate::wire::{Frame, Packet};

pub trait Handler {
  fn handle<'a>(
    &self,
    packet: Packet<'a>,
    frames: impl Iterator<Item = Result<Frame<'a>>>,
  ) -> impl Future<Output = Result<()>>;
}
