use std::io::Read;
use std::marker::PhantomData;

use crate::common::{ConnectionId, PacketNumber, VarInt};
use crate::crypto::Crypto;
use crate::packet::InitialPacket;
use crate::server::Server;
use quik_util::*;

pub struct Connection<S: Server>(PhantomData<S>);

impl<S: Server> Connection<S> {
    pub fn send(&self, data: &[u8]) {
        // Send data
    }

    pub fn recv(&self, mut data: &[u8]) -> Result<()> {
        let first_byte = data.read_u8()?;
        // Header Form (1) bit
        if first_byte >> 7 != 0 {
            // Long Header
            // Fixed Bit (1) = 1 - ignored
            // Long Packet Type (2) - not used in VersionNegotiation
            let packet_type = (first_byte >> 4) & 0b11;
            // Reserverd (2) - ignored
            // Packet Number Length (2) - not used in Retry & VersionNegotiation
            let packet_number_length = first_byte & 0b11;

            let version = data.read_u32::<NetworkEndian>()?;
            let dst_cid = ConnectionId::parse(&mut data)?;
            let src_cid = ConnectionId::parse(&mut data)?;

            if version == 0 {
                // VersionNegotiation packet
                return Ok(());
            }
            match packet_type {
                0b00 => {
                    // Initial packet
                    let token_length = VarInt::parse(&mut data)?;
                    let (token, mut data) = data
                        .split_at_checked(token_length.into())
                        .ok_or_else(|| "Token too long")?;
                    let packet_number =
                        PacketNumber::parse(&mut data, 1 + packet_number_length as usize)?;
                    self.handle_initial_packet(InitialPacket {
                        src_cid,
                        dst_cid: dst_cid.clone(),
                        version,
                        token,
                        packet_number,
                    })?;
                    let mut payload =
                        S::Crypto::decrypt_initial_data(dst_cid, version, false, &mut data)?;
                    // TODO there are multiple frames...
                    self.handle_raw_frame(&mut payload)?;
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

    fn handle_raw_frame(&self, data: &mut impl Buffer) -> Result<()> {
        todo!()
    }
}
