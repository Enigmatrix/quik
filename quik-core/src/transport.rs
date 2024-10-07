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
            data = self.parse_frame(data).await?;
        }
        Ok(())
    }

    async fn parse_frame<'a>(&'a self, mut data: &'a [u8]) -> Result<&'a [u8]> {
        let typ: usize = VarInt::parse(&mut data)?.into();
        match typ {
            0x00 => {
                // Padding
                // https://datatracker.ietf.org/doc/html/rfc9000#name-padding-frames
                self.handler.handle_frame(Frame::Padding).await?;
            }
            0x01 => {
                // Ping
                // https://datatracker.ietf.org/doc/html/rfc9000#name-ping-frames
                self.handler.handle_frame(Frame::Ping).await?;
            }
            0x02..=0x03 => {
                // Ack
                // https://datatracker.ietf.org/doc/html/rfc9000#name-ack-frames
                let largest_acked = VarInt::parse(&mut data)?;
                let ack_delay = VarInt::parse(&mut data)?;
                let ack_range_count = VarInt::parse(&mut data)?;
                let first_ack_range = VarInt::parse(&mut data)?;

                let ack_ranges = (0..ack_range_count.clone().into())
                    .map(|_| {
                        let gap = VarInt::parse(&mut data)?;
                        let range_length = VarInt::parse(&mut data)?;
                        Ok(frame::AckRange { gap, range_length })
                    })
                    .collect::<Result<Vec<_>>>()?;

                let ecn_counts = if typ == 0x03 {
                    let ect0 = VarInt::parse(&mut data)?;
                    let ect1 = VarInt::parse(&mut data)?;
                    let ce = VarInt::parse(&mut data)?;
                    Some(frame::EcnCounts { ect0, ect1, ce })
                } else {
                    None
                };

                self.handler
                    .handle_frame(Frame::Ack(frame::Ack {
                        largest_acked,
                        ack_delay,
                        ack_range_count,
                        first_ack_range,
                        ack_ranges,
                        ecn_counts,
                    }))
                    .await?;
            }
            0x04 => {
                // Reset Stream
                // https://datatracker.ietf.org/doc/html/rfc9000#name-reset_stream-frames
                let stream_id = VarInt::parse(&mut data)?;
                let err_code = VarInt::parse(&mut data)?;
                let final_size = VarInt::parse(&mut data)?;

                self.handler
                    .handle_frame(Frame::ResetStream(frame::ResetStream {
                        stream_id,
                        err_code,
                        final_size,
                    }))
                    .await?;
            }
            0x05 => {
                // Stop Sending
                // https://datatracker.ietf.org/doc/html/rfc9000#name-stop_sending-frames
                let stream_id = VarInt::parse(&mut data)?;
                let err_code = VarInt::parse(&mut data)?;

                self.handler
                    .handle_frame(Frame::StopSending(frame::StopSending {
                        stream_id,
                        err_code,
                    }))
                    .await?;
            }
            0x06 => {
                // Crypto
                // https://datatracker.ietf.org/doc/html/rfc9000#name-crypto-frames
                let offset = VarInt::parse(&mut data)?;
                let length = VarInt::parse(&mut data)?;
                let (_, rem_data) = data
                    .split_at_checked(offset.into())
                    .ok_or_else(|| "Not enough bytes for Crypto Offset")?;
                let (crypto_data, rem_data) = rem_data
                    .split_at_checked(length.into())
                    .ok_or_else(|| "Not enough bytes for Crypto Length")?;
                data = rem_data;

                self.handler
                    .handle_frame(Frame::Crypto(frame::Crypto { data: crypto_data }))
                    .await?;
            }
            0x07 => {
                // New Token
                // https://datatracker.ietf.org/doc/html/rfc9000#name-new_token-frames
                let token_length = VarInt::parse(&mut data)?;
                let (token, rem_data) = data
                    .split_at_checked(token_length.into())
                    .ok_or_else(|| "Not enough bytes for Token")?;
                data = rem_data;

                self.handler
                    .handle_frame(Frame::NewToken(frame::NewToken { token }))
                    .await?;
            }
            0x08..=0x0f => {
                // Stream
                // https://datatracker.ietf.org/doc/html/rfc9000#name-stream-frames
                let off_bit = typ & 0b100;
                let len_bit = typ & 0b010;
                let fin_bit = typ & 0b001;

                let stream_id = VarInt::parse(&mut data)?;
                let offset = if off_bit != 0 {
                    Some(VarInt::parse(&mut data)?)
                } else {
                    None
                };
                let length = if len_bit != 0 {
                    Some(VarInt::parse(&mut data)?)
                } else {
                    None
                };

                let (_, rem_data) = data
                    .split_at_checked(offset.unwrap_or(VarInt::ZERO).into())
                    .ok_or_else(|| "Not enough bytes for Stream Offset")?;
                let (stream_data, rem_data) = rem_data
                    .split_at_checked(length.unwrap_or(VarInt::ZERO).into())
                    .ok_or_else(|| "Not enough bytes for Stream Length")?;
                data = rem_data;

                self.handler
                    .handle_frame(Frame::Stream(frame::Stream {
                        stream_id,
                        data: stream_data,
                        fin: fin_bit != 0,
                    }))
                    .await?;
            }
            0x10 => {
                // Max Data
                // https://datatracker.ietf.org/doc/html/rfc9000#name-max_data-frames
                let max_data = VarInt::parse(&mut data)?;

                self.handler
                    .handle_frame(Frame::MaxData(frame::MaxData { max_data }))
                    .await?;
            }
            0x11 => {
                // Max Stream Data
                // https://datatracker.ietf.org/doc/html/rfc9000#name-max_stream_data-frames
                let stream_id = VarInt::parse(&mut data)?;
                let max_stream_data = VarInt::parse(&mut data)?;

                self.handler
                    .handle_frame(Frame::MaxStreamData(frame::MaxStreamData {
                        stream_id,
                        max_stream_data,
                    }))
                    .await?;
            }
            0x12..=0x13 => {
                // Max Streams
                // https://datatracker.ietf.org/doc/html/rfc9000#name-max_streams-frames
                let max_streams = VarInt::parse(&mut data)?;
                // TODO 0x12 is bidirectional, 0x13 is unidirectional. Does this matter?

                self.handler
                    .handle_frame(Frame::MaxStreams(frame::MaxStreams { max_streams }))
                    .await?;
            }
            0x14 => {
                // Data Blocked
                // https://datatracker.ietf.org/doc/html/rfc9000#name-data_blocked-frames
                let max_data = VarInt::parse(&mut data)?;

                self.handler
                    .handle_frame(Frame::DataBlocked(frame::DataBlocked { max_data }))
                    .await?;
            }
            0x15 => {
                // Stream Data Blocked
                // https://datatracker.ietf.org/doc/html/rfc9000#name-stream_data_blocked-frames
                let stream_id = VarInt::parse(&mut data)?;
                let max_stream_data = VarInt::parse(&mut data)?;

                self.handler
                    .handle_frame(Frame::StreamDataBlocked(frame::StreamDataBlocked {
                        stream_id,
                        max_stream_data,
                    }))
                    .await?;
            }
            0x16..=0x17 => {
                // Streams Blocked
                // https://datatracker.ietf.org/doc/html/rfc9000#name-streams_blocked-frames
                let max_streams = VarInt::parse(&mut data)?;
                // TODO 0x16 is bidirectional, 0x17 is unidirectional. Does this matter?

                self.handler
                    .handle_frame(Frame::StreamsBlocked(frame::StreamsBlocked { max_streams }))
                    .await?;
            }
            0x18 => {
                // New Connection ID
                // https://datatracker.ietf.org/doc/html/rfc9000#name-new_connection_id-frames
                let seq_num = VarInt::parse(&mut data)?;
                let retire_prior_to = VarInt::parse(&mut data)?;
                let cid = ConnectionId::parse(&mut data)?;
                let stateless_reset_token = data.read_u128::<NetworkEndian>()?;

                self.handler
                    .handle_frame(Frame::NewConnectionId(frame::NewConnectionId {
                        seq_num,
                        retire_prior_to,
                        cid,
                        stateless_reset_token,
                    }))
                    .await?;
            }
            0x19 => {
                // Retire Connection ID
                // https://datatracker.ietf.org/doc/html/rfc9000#name-retire_connection_id-frames
                let seq_num = VarInt::parse(&mut data)?;

                self.handler
                    .handle_frame(Frame::RetireConnectionId(frame::RetireConnectionId {
                        seq_num,
                    }))
                    .await?;
            }
            0x1a => {
                // Path Challenge
                // https://datatracker.ietf.org/doc/html/rfc9000#name-path_challenge-frames
                let data = data.read_u64::<NetworkEndian>()?;

                self.handler
                    .handle_frame(Frame::PathChallenge(frame::PathChallenge { data }))
                    .await?;
            }
            0x1b => {
                // Path Response
                // https://datatracker.ietf.org/doc/html/rfc9000#name-path_response-frames
                let data = data.read_u64::<NetworkEndian>()?;

                self.handler
                    .handle_frame(Frame::PathResponse(frame::PathResponse { data }))
                    .await?;
            }
            0x1c..=0x1d => {
                // Connection Close
                // https://datatracker.ietf.org/doc/html/rfc9000#name-connection_close-frames
                let err_code = VarInt::parse(&mut data)?;
                let frame_type = if typ == 0x1c {
                    Some(VarInt::parse(&mut data)?)
                } else {
                    None
                };
                let reason_phrase_length = VarInt::parse(&mut data)?;
                let (reason_phrase, rem_data) = data
                    .split_at_checked(reason_phrase_length.into())
                    .ok_or_else(|| "Not enough bytes for Reason Phrase")?;
                data = rem_data;

                self.handler
                    .handle_frame(Frame::ConnectionClose(frame::ConnectionClose {
                        err_code,
                        frame_type,
                        reason_phrase,
                    }))
                    .await?;
            }
            0x1e => {
                // Handshake Done
                // https://datatracker.ietf.org/doc/html/rfc9000#name-handshake_done-frames
                self.handler.handle_frame(Frame::HandshakeDone).await?;
            }
            _ => {
                Err("Unknown frame type")?;
            }
        }
        Ok(data)
    }
}
