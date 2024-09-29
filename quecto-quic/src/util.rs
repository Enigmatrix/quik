pub use byteorder::ReadBytesExt as Buffer;
pub use byteorder::{ByteOrder, NetworkEndian};

use std::error::Error;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
