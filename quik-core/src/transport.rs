use std::io;

use crate::common::{ConnectionId, PacketNumber, VarInt};
use crate::crypto::Crypto;
use crate::frame::{self, Frame};
use crate::packet::{self, Packet};
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
                    .map(|b| (&b[..]).read_u32::<NetworkEndian>())
                    .collect::<io::Result<Vec<u32>>>()?;
                // TODO check remainder?

                self.handler
                    .handle_packet(Packet::VersionNegotiation(packet::VersionNegotiation {
                        src_cid,
                        dst_cid: dst_cid.clone(),
                        supported_versions: &versions,
                    }))
                    .await?;
                return Ok(());
            }
            match packet_type {
                0b00 => {
                    // Initial packet
                    // https://datatracker.ietf.org/doc/html/rfc9000#name-initial-packet

                    let token_length = VarInt::parse(&mut data)?;
                    let (token, rem_data) = data
                        .split_at_checked(token_length.into())
                        .ok_or_else(|| "Token too long")?;
                    data = rem_data;
                    let length = VarInt::parse(&mut data)?; // TODO use this
                    let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;

                    self.handler
                        .handle_packet(Packet::Initial(packet::Initial {
                            src_cid,
                            dst_cid: dst_cid.clone(),
                            version,
                            token,
                            packet_number,
                        }))
                        .await?;
                    let mut payload = self
                        .crypto
                        .decrypt_initial_data(dst_cid, version, false, &mut data)
                        .await?;
                    self.handle_payload(&mut payload).await?;
                }
                0b01 => {
                    // 0-RTT packet
                    // https://datatracker.ietf.org/doc/html/rfc9000#name-0-rtt

                    let length = VarInt::parse(&mut data)?; // TODO use this
                    let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
                    let mut payload = data; // TODO: decrypt

                    self.handler
                        .handle_packet(Packet::ZeroRTT(packet::ZeroRTT {
                            src_cid,
                            dst_cid: dst_cid.clone(),
                            version,
                            packet_number,
                        }))
                        .await?;
                    self.handle_payload(&mut payload).await?;
                }
                0b10 => {
                    // Handshake packet
                    // https://datatracker.ietf.org/doc/html/rfc9000#packet-handshake

                    let length = VarInt::parse(&mut data)?; // TODO use this
                    let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
                    let mut payload = data; // TODO: decrypt

                    self.handler
                        .handle_packet(Packet::Handshake(packet::Handshake {
                            src_cid,
                            dst_cid: dst_cid.clone(),
                            version,
                            packet_number,
                        }))
                        .await?;
                    self.handle_payload(&mut payload).await?;
                }
                0b11 => {
                    // Retry packet
                    // https://datatracker.ietf.org/doc/html/rfc9000#name-retry-packet

                    // TODO is this encoding even correct? wtf is a retry token?
                    let (retry_token, retry_integrity_tag) = data
                        .split_last_chunk::<16>() // 128bits/8 = 16 bytes
                        .ok_or_else(|| "Packet too short for Retry Token")?;
                    let retry_integrity_tag =
                        (&retry_integrity_tag[..]).read_u128::<NetworkEndian>()?;

                    self.handler
                        .handle_packet(Packet::Retry(packet::Retry {
                            src_cid,
                            dst_cid: dst_cid.clone(),
                            version,
                            retry_token,
                            retry_integrity_tag,
                        }))
                        .await?;
                }
                _ => unreachable!(),
            }
        } else {
            // Short Header
            // https://datatracker.ietf.org/doc/html/rfc9000#name-short-header-packets

            // Fixed Bit (1) = 1 - ignored
            // Spin Bit (1)
            let spin = (first_byte >> 5) & 1;
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

            self.handler
                .handle_packet(Packet::OneRtt(packet::OneRtt {
                    dst_cid: dst_cid.clone(),
                    packet_number,
                    spin,
                    key_phase,
                }))
                .await?;
            self.handle_payload(&mut payload).await?;
        }
        Ok(())
    }

    pub fn close(self) {
        // Close connection
    }

    async fn handle_payload(&self, mut data: &[u8]) -> Result<()> {
        while !data.is_empty() {
            let frame;
            (frame, data) = Frame::parse(data).await?;
            self.handler.handle_frame(frame).await?;
        }
        Ok(())
    }
}
