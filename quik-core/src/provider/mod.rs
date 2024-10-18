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
    sid: StreamId,
  ) -> impl Future<Output = Result<Self::StreamRx>>;
}

pub struct DefaultProvider {
  cb: Box<
    dyn Fn(
      &mut <DefaultProvider as Provider>::Connection,
      &<DefaultProvider as Provider>::StreamRx,
    ) -> Result<()>,
  >,
}

impl DefaultProvider {
  pub fn new(
    cb: Box<
      dyn Fn(
        &mut <DefaultProvider as Provider>::Connection,
        &<DefaultProvider as Provider>::StreamRx,
      ) -> Result<()>,
    >,
  ) -> Self {
    Self { cb: Box::new(cb) }
  }
}

impl Provider for DefaultProvider {
  type Connection = DefaultConnection;
  type StreamRx = DefaultStreamRx;

  async fn create_connection(&self, cid: ConnectionId) -> Result<Self::Connection> {
    Ok(DefaultConnection::new(cid))
  }

  async fn create_stream(
    &self,
    conn: &mut Self::Connection,
    sid: StreamId,
  ) -> Result<Self::StreamRx> {
    let rx = DefaultStreamRx::new(sid, None);
    (self.cb)(conn, &rx)?;
    Ok(rx)
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::*;

  #[tokio::test]
  async fn provider_creation() -> Result<()> {
    let called = Arc::new(Mutex::new(false));
    let _called = called.clone();

    let provider = DefaultProvider::new(Box::new(move |_, _| {
      let called = called.clone();
      tokio::spawn(async move {
        *called.lock().await = true;
      });
      Ok(())
    }));
    
    let conn = &mut DefaultConnection::new(ConnectionId::parse(&mut (&[1, 0x12][..]))?);
    let sid = StreamId::parse(&mut (&[0x12][..]))?;
    provider.create_stream(conn, sid).await?;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert_eq!(*(_called.lock().await), true);
    Ok(())
  }
}
