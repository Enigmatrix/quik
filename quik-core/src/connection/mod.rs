use std::future::Future;

use quik_util::*;

trait Connection {
  fn dropped(&self) -> impl Future<Output = Result<()>>;
}
