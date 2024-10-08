use quik_util::*;

use crate::common::ConnectionId;

pub trait Crypto {
    fn decrypt_initial_data(
        &self,
        cid: ConnectionId,
        version: u32,
        is_server: bool,
        data: &mut impl Buffer,
    ) -> impl Future<Output = Result<Vec<u8>>>;
}
