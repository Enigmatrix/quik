use std::future::Future;

use quik_util::*;

use crate::connection::{Connection, DefaultConnection};
use crate::stream::{DefaultStreamRx, StreamRx};
use crate::wire::{ConnectionId, StreamId};

pub trait Provider {
  type Connection: Connection;
  type StreamRx: StreamRx;

  fn create_connection(&self, cid: ConnectionId) -> impl Future<Output = Result<Self::Connection>>;
  fn create_stream(
    &self,
    conn: &mut Self::Connection,
    id: StreamId,
  ) -> impl Future<Output = Result<Self::StreamRx>>;
}

pub struct DefaultProvider;

impl Provider for DefaultProvider {
  type Connection = DefaultConnection;
  type StreamRx = DefaultStreamRx;

  async fn create_connection(&self, cid: ConnectionId) -> Result<Self::Connection> {
    Ok(DefaultConnection::new(cid))
  }

  async fn create_stream(
    &self,
    conn: &mut Self::Connection,
    id: StreamId,
  ) -> Result<Self::StreamRx> {
    let rx = DefaultStreamRx::new(None);
    Ok(rx)
  }
}
