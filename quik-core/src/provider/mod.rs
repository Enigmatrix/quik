use std::future::Future;

use quik_util::*;

use crate::connection::Connection;
use crate::stream::StreamRx;
use crate::wire::{ConnectionId, StreamId};

trait Provider {
  type Connection: Connection;
  type StreamRx: StreamRx;

  fn create_connection(&self, id: ConnectionId) -> impl Future<Output = Result<Self::Connection>>;
  fn create_stream(
    &self,
    conn: &mut Self::Connection,
    id: StreamId,
  ) -> impl Future<Output = Result<Self::StreamRx>>;
}
