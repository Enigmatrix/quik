use quik_core::common::ConnectionId;
use quik_core::crypto::Crypto;
use quik_util::*;

pub struct DefaultCrypto;

impl Crypto for DefaultCrypto {
    fn decrypt_initial_data(
        cid: ConnectionId,
        version: u32,
        is_server: bool,
        data: &[u8],
    ) -> Result<impl Buffer> {
        todo!();
        Ok(&[][..])
    }
}
