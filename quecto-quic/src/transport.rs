use std::io::Read;

use crate::{
    frame::{PacketNumber, VarInt},
    packet::{Buffer, InitialPacket},
};
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
            let dest_conn_id = ConnectionId::parse(&mut data)?;
            let src_conn_id = ConnectionId::parse(&mut data)?;
            if (first_byte >> 1) & 0b1 == 0 {
                if version == 0 {
                    // VersionNegotiation packet
                } else {
                    // throw error, this is invalid
                }
            }
            let packet_number_length = first_byte >> 6; // not always needed
            let packet_type = (first_byte >> 2) & 0b11;
            match packet_type {
                0b00 => {
                    // Initial packet
                    let token_length = VarInt::parse(&mut data)?;
                    let (token, mut data) = data
                        .split_at_checked(token_length.into())
                        .ok_or("Token too long")?;
                    let packet_number =
                        PacketNumber::parse(&mut data, 1 + packet_number_length as usize)?;
                    self.handle_initial_packet(InitialPacket {
                        src_conn_id,
                        dest_conn_id,
                        version,
                        token,
                        packet_number,
                    })?;
                    self.handle_raw_frame(data)?;
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

    fn handle_initial_packet(&self, packet: InitialPacket<'_>) -> Result<()> {
        todo!()
    }

    fn handle_raw_frame(&self, data: &[u8]) -> Result<()> {
        todo!()
    }
}
