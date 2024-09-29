use crate::util::*;

pub struct PacketNumber;

impl PacketNumber {
    pub fn parse(src: &mut impl Buffer, len: usize) -> Result<u32> {
        let mut buf = [0u8; 4];
        src.read_exact(&mut buf[..len])?;
        Ok(NetworkEndian::read_u32(&buf))
    }
}

pub struct VarInt {
    inner: u64,
}

impl From<VarInt> for usize {
    fn from(val: VarInt) -> Self {
        val.inner as usize
    }
}

impl VarInt {
    pub fn parse(src: &mut impl Buffer) -> Result<Self> {
        let mut buf = [0u8; 8];
        src.read_exact(&mut buf[..1])?;
        let typ = buf[0] >> 6;
        buf[0] &= 0b0011_1111;

        Ok(Self {
            inner: match typ {
                0b00 => buf[0] as u64,
                0b01 => {
                    src.read_exact(&mut buf[1..2])?;
                    NetworkEndian::read_u16(&buf) as u64
                }
                0b10 => {
                    src.read_exact(&mut buf[1..4])?;
                    NetworkEndian::read_u32(&buf) as u64
                }
                0b11 => {
                    src.read_exact(&mut buf[1..8])?;
                    NetworkEndian::read_u64(&buf) as u64
                }
                _ => unreachable!(),
            },
        })
    }
}

pub type StreamId = VarInt;

// 160 bits max, variable length
pub struct ConnectionId {
    pub length: usize,
    pub buf: [u8; 20],
}

impl ConnectionId {
    pub fn parse(src: &mut impl Buffer) -> Result<Self> {
        let length = src.read_u8()? as usize;
        let mut buf = [0; 20];
        src.read_exact(&mut buf[..length])?;
        Ok(Self { length, buf })
    }
}
