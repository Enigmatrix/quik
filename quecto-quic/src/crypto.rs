use quecto_util::*;

pub trait Crypto {
    fn decrypt_initial_data(cid: &[u8], version: u32, is_server: bool, data: &[u8]) -> Result<Vec<u8>>;
}