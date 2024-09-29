use quecto_quic::crypto::Crypto;
use quecto_util::*;

pub struct DefaultCrypto;

impl Crypto for DefaultCrypto {
    fn decrypt_initial_data(cid: &[u8], version: u32, is_server: bool, data: &[u8]) -> Result<Vec<u8>> {
        todo!()
    }
}