use crate::common::ConnectionId;
use quik_util::*;

pub trait Crypto {
    fn decrypt_initial_data(
        cid: ConnectionId,
        version: u32,
        is_server: bool,
        data: &mut impl Buffer,
    ) -> Result<impl Buffer>;
}
