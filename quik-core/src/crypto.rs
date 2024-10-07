use crate::common::ConnectionId;
use quik_util::*;

pub trait Crypto {
    fn decrypt_initial_data(
        &self,
        cid: ConnectionId,
        version: u32,
        is_server: bool,
        data: &mut impl Buffer,
    ) -> impl Future<Output = Result<Vec<u8>>>;
}
