use crate::common::{ConnectionId, PacketNumber, VarInt};
use crate::crypto::Crypto;
use crate::packet::{self, InitialPacket};
use crate::server::Handler;
use quik_util::*;

pub trait Io {
    fn send(&self, data: &[u8]) -> impl Future<Output = Result<()>>;
    fn recv(&self, data: &mut [u8]) -> impl Future<Output = Result<()>>;
    fn close(self) -> impl Future<Output = ()>;
}

pub struct Connection<C: Crypto, I: Io, H: Handler> {
    crypto: C,
    io: I,
    handler: H,
}

impl<C: Crypto, I: Io, H: Handler> Connection<C, I, H> {
    pub fn send(&self, data: &[u8]) {
        // Send data
    }

    pub async fn recv(&self, mut data: &[u8]) -> Result<()> {
        let first_byte = data.read_u8()?;
        // Header Form (1) bit
        if first_byte >> 7 != 0 {
            // Long Header
            // https://datatracker.ietf.org/doc/html/rfc9000#long-header

            // Fixed Bit (1) = 1 - ignored
            // Long Packet Type (2) - not used in VersionNegotiation
            let packet_type = (first_byte >> 4) & 0b11;
            // Reserved (2) - ignored
            // Packet Number Length (2) - not used in Retry & VersionNegotiation
            let packet_number_length_encoded = first_byte & 0b11;
            let packet_number_length = 1 + packet_number_length_encoded as usize;

            let version = data.read_u32::<NetworkEndian>()?;
            let dst_cid = ConnectionId::parse(&mut data)?;
            let src_cid = ConnectionId::parse(&mut data)?;

            if version == 0 {
                // VersionNegotiation packet
                // https://datatracker.ietf.org/doc/html/rfc9000#name-version-negotiation-packet
                let (versions_bytes, _remainder) = data.as_chunks::<4>();
                let versions = versions_bytes
                    .iter()
                    .map(|b| (&b[..]).read_u32::<NetworkEndian>());
                // TODO handle version negotiation
                return Ok(());
            }
            match packet_type {
                0b00 => {
                    // Initial packet
                    let token_length = VarInt::parse(&mut data)?;
                    let (token, mut data) = data
                        .split_at_checked(token_length.into())
                        .ok_or_else(|| "Token too long")?;
                    let length = VarInt::parse(&mut data)?; // TODO use this
                    let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
                    self.handler
                        .handle_initial_packet(InitialPacket {
                            src_cid,
                            dst_cid: dst_cid.clone(),
                            version,
                            token,
                            packet_number,
                        })
                        .await?;
                    let mut payload = self
                        .crypto
                        .decrypt_initial_data(dst_cid, version, false, &mut data)
                        .await?;
                    self.handle_payload(&mut payload)?;
                }
                0b01 => {
                    // 0-RTT packet
                    // https://datatracker.ietf.org/doc/html/rfc9000#name-0-rtt
                    let length = VarInt::parse(&mut data)?; // TODO use this
                    let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
                    let mut payload = data; // TODO: decrypt
                                            // TODO handle 0-rtt
                    self.handle_payload(&mut payload)?;
                }
                0b10 => {
                    // Handshake packet
                    // https://datatracker.ietf.org/doc/html/rfc9000#packet-handshake
                    let length = VarInt::parse(&mut data)?; // TODO use this
                    let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
                    let mut payload = data; // TODO: decrypt
                                            // TODO handle handshake packet
                    self.handle_payload(&mut payload)?;
                }
                0b11 => {
                    // Retry packet
                    // https://datatracker.ietf.org/doc/html/rfc9000#name-retry-packet
                    // TODO is this encoding even correct?
                    let (retry_token, retry_integrity_tag) = data
                        .split_last_chunk::<16>() // 128bits/8 = 16 bytes
                        .ok_or_else(|| "Packet too short for Retry Token")?;
                    // TODO handle retry
                }
                _ => unreachable!(),
            }
        } else {
            // Short Header
            // https://datatracker.ietf.org/doc/html/rfc9000#name-short-header-packets

            // Fixed Bit (1) = 1 - ignored
            // Spin Bit (1)
            let spin_bit = (first_byte >> 5) & 1;
            // Reserved (2) - ignored
            // Key Phase (1)
            let key_phase = (first_byte >> 2) & 1;
            // Packet Number Length (2)
            let packet_number_length_encoded = first_byte & 0b11;
            let packet_number_length = 1 + packet_number_length_encoded as usize;
            let dst_cid = ConnectionId::parse(&mut data)?;

            // Currently 1-RTT packets are the only Short Header packets
            // https://datatracker.ietf.org/doc/html/rfc9000#name-1-rtt-packet
            let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
            let mut payload = data; // TODO: decrypt
                                    // TODO handle 1-rtt
            self.handle_payload(&mut payload)?;
        }
        Ok(())
    }

    pub fn close(self) {
        // Close connection
    }

    fn handle_payload(&self, data: &mut impl Buffer) -> Result<()> {
        todo!()
    }
}
