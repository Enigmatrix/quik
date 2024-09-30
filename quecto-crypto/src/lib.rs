use quecto_quic::common::ConnectionId;
use quecto_quic::crypto::Crypto;
use quecto_util::*;

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
