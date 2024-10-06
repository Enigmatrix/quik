pub use byteorder::ReadBytesExt;
pub use byteorder::{ByteOrder, NetworkEndian};

pub use std::future::*;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Buffer: ReadBytesExt {
    fn slice(&mut self, len: usize) -> Result<Self>
    where
        Self: Sized;
    fn extract(&mut self, off: Option<usize>, len: Option<usize>) -> Result<Self>
    where
        Self: Sized;
}

impl Buffer for &[u8] {
    fn slice(&mut self, len: usize) -> Result<Self> {
        let ret;
        (ret, *self) = self
            .split_at_checked(len)
            .ok_or_else(|| "Buffer too short")?;
        Ok(ret)
    }

    fn extract(&mut self, off: Option<usize>, len: Option<usize>) -> Result<Self> {
        let ret;
        if let Some(off) = off {
            (_, *self) = self
                .split_at_checked(off)
                .ok_or_else(|| "Buffer too short for offset")?;
        }
        if let Some(len) = len {
            (ret, *self) = self
                .split_at_checked(len)
                .ok_or_else(|| "Buffer too short for length")?;
            return Ok(ret);
        }
        let data = *self;
        *self = &[];
        Ok(data)
    }
}
