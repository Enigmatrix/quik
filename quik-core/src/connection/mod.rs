use std::future::Future;

use quik_util::*;

use crate::wire::ConnectionId;

pub trait Connection {
  fn dropped(&self) -> impl Future<Output = Result<()>>;
}

pub struct DefaultConnection {
  cid: ConnectionId,
}

impl DefaultConnection {
  pub fn new(cid: ConnectionId) -> Self {
    Self { cid }
  }
}

impl Connection for DefaultConnection {
  async fn dropped(&self) -> Result<()> {
    // Do nothing!
    Ok(())
  }
}
