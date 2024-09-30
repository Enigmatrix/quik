use crate::common::ConnectionId;
use quik_util::*;

pub trait Crypto {
    fn decrypt_initial_data(
        cid: ConnectionId,
        version: u32,
        is_server: bool,
        data: &[u8],
    ) -> Result<impl Buffer>;
}
