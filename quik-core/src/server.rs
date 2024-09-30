use crate::crypto::Crypto;

pub trait Server {
    type Crypto: Crypto;
}
