pub use byteorder::ReadBytesExt as Buffer;
pub use byteorder::{ByteOrder, NetworkEndian};

pub use std::future::*;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
