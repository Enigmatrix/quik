use quecto_util::*;
use std::io;

pub struct PacketNumber;

impl PacketNumber {
    // Assumes length has correct bounds, or panics
    pub fn parse(src: &mut impl Buffer, len: usize) -> Result<u32> {
        let mut buf = [0u8; 4];
        src.read_exact(&mut buf[..len])?;
        Ok(NetworkEndian::read_u32(&buf))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConnectionId {
    pub length: usize,
    pub buf: [u8; 20],
}

impl ConnectionId {
    pub fn parse(src: &mut impl Buffer) -> Result<Self> {
        let length = src.read_u8()? as usize;
        let mut buf = [0; 20];
        let (bufref, _) = buf.split_at_mut_checked(length).ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "length longer than 160 bits")
        })?;
        src.read_exact(bufref)?;
        Ok(Self { length, buf })
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn varint_parse_no_bytes_fails() {
        let buf = Vec::<u8>::new();
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn varint_parse_one_byte_succeeds() {
        let buf = vec![0b0011_0101, 0x34];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        assert_eq!(res.ok(), Some(VarInt { inner: 0b0011_0101 }));
        assert_eq!(bufref, &[0x34]);
    }

    #[test]
    fn varint_parse_two_byte_succeeds() {
        let buf = vec![0b0110_0101, 0x34, 0x12];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        assert_eq!(res.ok(), Some(VarInt { inner: 0x2534 }));
        assert_eq!(bufref, &[0x12]);
    }

    #[test]
    fn varint_parse_four_byte_succeeds() {
        let buf = vec![0b1010_0101, 0x12, 0x34, 0x56, 0x78];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        assert_eq!(res.ok(), Some(VarInt { inner: 0x25123456 }));
        assert_eq!(bufref, &[0x78]);
    }

    #[test]
    fn varint_parse_eight_byte_succeeds() {
        let buf = vec![0b1110_0101, 0x12, 0x34, 0x56, 0x78, 0x90, 0x11, 0x22, 0xaa];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        assert_eq!(
            res.ok(),
            Some(VarInt {
                inner: 0x2512345678901122
            })
        );
        assert_eq!(bufref, &[0xaa]);
    }

    #[test]
    fn varint_parse_one_byte_size_fails() {
        let buf = vec![0b0100_0000];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));

        let buf = vec![0b1000_0000];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));

        let buf = vec![0b1100_0000];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn varint_parse_two_byte_fails() {
        let buf = vec![0b0110_0101];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn varint_parse_four_byte_fails() {
        let buf = vec![0b1010_0101, 0x12, 0x34];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn varint_parse_eight_byte_fails() {
        let buf = vec![0b1110_0101, 0x12, 0x34, 0x56, 0x78, 0x90, 0x11];
        let mut bufref = &buf[..];
        let res = VarInt::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn connid_parse_no_bytes_fails() {
        let buf = Vec::<u8>::new();
        let mut bufref = &buf[..];
        let res = ConnectionId::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn connid_parse_size_only_fails() {
        let buf = vec![19u8];
        let mut bufref = &buf[..];
        let res = ConnectionId::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn connid_parse_size_big_fails() {
        let buf = vec![20u8];
        let mut bufref = &buf[..];
        let res = ConnectionId::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn connid_parse_size_very_big_fails() {
        let buf = vec![0xffu8];
        let mut bufref = &buf[..];
        let res = ConnectionId::parse(&mut bufref);
        let kind = res
            .err()
            .as_ref()
            .and_then(|e| e.downcast_ref::<io::Error>())
            .map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn connid_parse_one_byte_passes() {
        let buf = vec![1u8, 0x12, 0x34];
        let mut bufref = &buf[..];
        let res = ConnectionId::parse(&mut bufref);
        assert_eq!(
            res.ok(),
            Some(ConnectionId {
                length: 1 as usize,
                buf: [0x12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            })
        );
        assert_eq!(bufref, &[0x34])
    }

    #[test]
    fn connid_parse_10_bytes_passes() {
        let buf = vec![
            10u8, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0x12, 0x34,
        ];
        let mut bufref = &buf[..];
        let res = ConnectionId::parse(&mut bufref);
        assert_eq!(
            res.ok(),
            Some(ConnectionId {
                length: 10 as usize,
                buf: [
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0
                ]
            })
        );
        assert_eq!(bufref, &[0x12, 0x34])
    }

    #[test]
    fn connid_parse_max_byte_passes() {
        let buf = vec![
            20u8, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0x22, 0x11, 0x33,
            0x55, 0x44, 0x77, 0x66, 0x99, 0x11, 0x20, 0x12, 0x34,
        ];
        let mut bufref = &buf[..];
        let res = ConnectionId::parse(&mut bufref);
        assert_eq!(
            res.ok(),
            Some(ConnectionId {
                length: 20 as usize,
                buf: [
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0x22, 0x11, 0x33,
                    0x55, 0x44, 0x77, 0x66, 0x99, 0x11, 0x20
                ]
            })
        );
        assert_eq!(bufref, &[0x12, 0x34])
    }
}
