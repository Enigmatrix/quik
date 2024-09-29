use qwik_core::common::ConnectionId;
use qwik_core::crypto::Crypto;
use qwik_util::*;

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
