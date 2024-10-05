use crate::crypto::Crypto;
use crate::transport::Io;

pub trait Server {
    type Crypto: Crypto;
    type Io: Io;
}
