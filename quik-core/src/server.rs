use quik_util::*;

use crate::crypto::Crypto;
use crate::handler::Handler;
use crate::transport::Io;

pub trait Server {
  type Crypto: Crypto;
  // UDP transport
  type Io: Io;
  type Handler: Handler;
}
