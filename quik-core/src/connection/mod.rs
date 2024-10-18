use std::future::Future;

use quik_util::*;

pub trait Connection {
  fn dropped(&self) -> impl Future<Output = Result<()>>;
}
