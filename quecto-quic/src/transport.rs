use crate::packet::Buffer;
use byteorder::NetworkEndian;

use crate::packet::{ConnectionId, Result};

pub struct Connection;

impl Connection {
    pub fn send(&self, data: &[u8]) {
        // Send data
    }

    pub fn recv(&self, mut data: &[u8]) -> Result<()> {
        let first_byte = data.read_u8()?;
        if first_byte & 0b1 == 1 {
            // Long Header
            let version = data.read_u32::<NetworkEndian>()?;
            let dest_conn_id = ConnectionId::parse(data)?;
            let src_conn_id = ConnectionId::parse(data)?;
            if (first_byte >> 1) & 0b1 == 0 {
                if version == 0 {
                    // VersionNegotiation packet
                } else {
                    // throw error, this is invalid
                }
            }
            let packet_type = (first_byte >> 2) & 0b11;
            match packet_type {
                0b00 => {
                    // Initial packet
                }
                0b01 => {
                    // 0-RTT packet
                }
                0b10 => {
                    // Handshake packet
                }
                0b11 => {
                    // Retry packet
                }
                _ => unreachable!(),
            }
        } else {
            // Short Header
        }
        Ok(())
    }

    pub fn close(self) {
        // Close connection
    }
}
