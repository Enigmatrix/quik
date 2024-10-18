use quik_util::*;

use crate::wire::{ConnectionId, StreamId, VarInt};

pub enum Frame<'a> {
    Padding,
    Ping,
    Ack(Ack),
    ResetStream(ResetStream),
    StopSending(StopSending),
    Crypto(Crypto<'a>),
    NewToken(NewToken<'a>),
    Stream(Stream<'a>),
    MaxData(MaxData),
    MaxStreamData(MaxStreamData),
    MaxStreams(MaxStreams),
    DataBlocked(DataBlocked),
    StreamDataBlocked(StreamDataBlocked),
    StreamsBlocked(StreamsBlocked),
    NewConnectionId(NewConnectionId),
    RetireConnectionId(RetireConnectionId),
    PathChallenge(PathChallenge),
    PathResponse(PathResponse),
    ConnectionClose(ConnectionClose<'a>),
    HandshakeDone,
}

pub struct Padding;

pub struct Ping;

pub struct AckRange {
    pub gap: VarInt,
    pub range_length: VarInt,
}

pub struct EcnCounts {
    pub ect0: VarInt,
    pub ect1: VarInt,
    pub ce: VarInt,
}

pub struct Ack {
    pub largest_acked: VarInt,
    pub ack_delay: VarInt,
    pub ack_range_count: VarInt,
    pub first_ack_range: VarInt,
    pub ack_ranges: Vec<AckRange>,
    pub ecn_counts: Option<EcnCounts>,
}

pub struct ResetStream {
    pub stream_id: StreamId,
    pub err_code: VarInt,
    pub final_size: VarInt,
}

pub struct StopSending {
    pub stream_id: StreamId,
    pub err_code: VarInt,
}

pub struct Crypto<'a> {
    pub data: &'a [u8],
}

pub struct NewToken<'a> {
    pub token: &'a [u8],
}

pub struct Stream<'a> {
    pub stream_id: StreamId,
    pub fin: bool,
    pub data: &'a [u8],
}

pub struct MaxData {
    pub max_data: VarInt,
}

pub struct MaxStreamData {
    pub stream_id: StreamId,
    pub max_stream_data: VarInt,
}

pub struct MaxStreams {
    pub max_streams: VarInt,
}

pub struct DataBlocked {
    pub max_data: VarInt,
}

pub struct StreamDataBlocked {
    pub stream_id: StreamId,
    pub max_stream_data: VarInt,
}

pub struct StreamsBlocked {
    pub max_streams: VarInt,
}

pub struct NewConnectionId {
    pub seq_num: VarInt,
    pub retire_prior_to: VarInt,
    pub cid: ConnectionId,
    pub stateless_reset_token: u128,
}

pub struct RetireConnectionId {
    pub seq_num: VarInt,
}

pub struct PathChallenge {
    pub data: u64,
}

pub struct PathResponse {
    pub data: u64,
}

pub struct ConnectionClose<'a> {
    pub err_code: VarInt,
    // Some when it's a QUIC err rather than a application error
    pub frame_type: Option<VarInt>,
    pub reason_phrase: &'a [u8],
}

pub struct HandshakeDone;

impl<'a> Frame<'a> {
    pub fn parse_multiple(mut data: &'a [u8]) -> impl Iterator<Item = Result<Frame<'a>>> {
        std::iter::from_fn(move || {
            if !data.is_empty() {
                Some(Frame::parse(data).map(|(frame, rem_data)| {
                    data = rem_data;
                    frame
                }))
            } else {
                None
            }
        })
    }

    pub fn parse(mut data: &'a [u8]) -> Result<(Frame<'a>, &'a [u8])> {
        let typ: usize = VarInt::parse(&mut data)?.into();
        let frame = match typ {
            0x00 => {
                // Padding
                // https://datatracker.ietf.org/doc/html/rfc9000#name-padding-frames
                Frame::Padding
            }
            0x01 => {
                // Ping
                // https://datatracker.ietf.org/doc/html/rfc9000#name-ping-frames
                Frame::Ping
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
                        Ok(AckRange { gap, range_length })
                    })
                    .collect::<Result<Vec<_>>>()?;

                let ecn_counts = if typ == 0x03 {
                    let ect0 = VarInt::parse(&mut data)?;
                    let ect1 = VarInt::parse(&mut data)?;
                    let ce = VarInt::parse(&mut data)?;
                    Some(EcnCounts { ect0, ect1, ce })
                } else {
                    None
                };

                Frame::Ack(Ack {
                    largest_acked,
                    ack_delay,
                    ack_range_count,
                    first_ack_range,
                    ack_ranges,
                    ecn_counts,
                })
            }
            0x04 => {
                // Reset Stream
                // https://datatracker.ietf.org/doc/html/rfc9000#name-reset_stream-frames
                let stream_id = VarInt::parse(&mut data)?;
                let err_code = VarInt::parse(&mut data)?;
                let final_size = VarInt::parse(&mut data)?;

                Frame::ResetStream(ResetStream {
                    stream_id,
                    err_code,
                    final_size,
                })
            }
            0x05 => {
                // Stop Sending
                // https://datatracker.ietf.org/doc/html/rfc9000#name-stop_sending-frames
                let stream_id = VarInt::parse(&mut data)?;
                let err_code = VarInt::parse(&mut data)?;

                Frame::StopSending(StopSending {
                    stream_id,
                    err_code,
                })
            }
            0x06 => {
                // Crypto
                // https://datatracker.ietf.org/doc/html/rfc9000#name-crypto-frames
                let offset = VarInt::parse(&mut data)?;
                let length = VarInt::parse(&mut data)?;
                let crypto_data = data.extract(Some(offset.into()), Some(length.into()))?;

                Frame::Crypto(Crypto { data: crypto_data })
            }
            0x07 => {
                // New Token
                // https://datatracker.ietf.org/doc/html/rfc9000#name-new_token-frames
                let token_length = VarInt::parse(&mut data)?;
                let token = data.slice(token_length.into())?;

                Frame::NewToken(NewToken { token })
            }
            0x08..=0x0f => {
                // Stream
                // https://datatracker.ietf.org/doc/html/rfc9000#name-stream-frames
                let off_bit = typ & 0b100;
                let len_bit = typ & 0b010;
                let fin_bit = typ & 0b001;

                let stream_id = VarInt::parse(&mut data)?;
                let offset = if off_bit != 0 {
                    Some(VarInt::parse(&mut data)?.into())
                } else {
                    None
                };
                let length = if len_bit != 0 {
                    Some(VarInt::parse(&mut data)?.into())
                } else {
                    None
                };
                let stream_data = data.extract(offset.into(), length.into())?;

                Frame::Stream(Stream {
                    stream_id,
                    data: stream_data,
                    fin: fin_bit != 0,
                })
            }
            0x10 => {
                // Max Data
                // https://datatracker.ietf.org/doc/html/rfc9000#name-max_data-frames
                let max_data = VarInt::parse(&mut data)?;

                Frame::MaxData(MaxData { max_data })
            }
            0x11 => {
                // Max Stream Data
                // https://datatracker.ietf.org/doc/html/rfc9000#name-max_stream_data-frames
                let stream_id = VarInt::parse(&mut data)?;
                let max_stream_data = VarInt::parse(&mut data)?;

                Frame::MaxStreamData(MaxStreamData {
                    stream_id,
                    max_stream_data,
                })
            }
            0x12..=0x13 => {
                // Max Streams
                // https://datatracker.ietf.org/doc/html/rfc9000#name-max_streams-frames
                let max_streams = VarInt::parse(&mut data)?;
                // TODO 0x12 is bidirectional, 0x13 is unidirectional. Does this matter?

                Frame::MaxStreams(MaxStreams { max_streams })
            }
            0x14 => {
                // Data Blocked
                // https://datatracker.ietf.org/doc/html/rfc9000#name-data_blocked-frames
                let max_data = VarInt::parse(&mut data)?;

                Frame::DataBlocked(DataBlocked { max_data })
            }
            0x15 => {
                // Stream Data Blocked
                // https://datatracker.ietf.org/doc/html/rfc9000#name-stream_data_blocked-frames
                let stream_id = VarInt::parse(&mut data)?;
                let max_stream_data = VarInt::parse(&mut data)?;

                Frame::StreamDataBlocked(StreamDataBlocked {
                    stream_id,
                    max_stream_data,
                })
            }
            0x16..=0x17 => {
                // Streams Blocked
                // https://datatracker.ietf.org/doc/html/rfc9000#name-streams_blocked-frames
                let max_streams = VarInt::parse(&mut data)?;
                // TODO 0x16 is bidirectional, 0x17 is unidirectional. Does this matter?

                Frame::StreamsBlocked(StreamsBlocked { max_streams })
            }
            0x18 => {
                // New Connection ID
                // https://datatracker.ietf.org/doc/html/rfc9000#name-new_connection_id-frames
                let seq_num = VarInt::parse(&mut data)?;
                let retire_prior_to = VarInt::parse(&mut data)?;
                let cid = ConnectionId::parse(&mut data)?;
                let stateless_reset_token = data.read_u128::<NetworkEndian>()?;

                Frame::NewConnectionId(NewConnectionId {
                    seq_num,
                    retire_prior_to,
                    cid,
                    stateless_reset_token,
                })
            }
            0x19 => {
                // Retire Connection ID
                // https://datatracker.ietf.org/doc/html/rfc9000#name-retire_connection_id-frames
                let seq_num = VarInt::parse(&mut data)?;

                Frame::RetireConnectionId(RetireConnectionId { seq_num })
            }
            0x1a => {
                // Path Challenge
                // https://datatracker.ietf.org/doc/html/rfc9000#name-path_challenge-frames
                let data = data.read_u64::<NetworkEndian>()?;

                Frame::PathChallenge(PathChallenge { data })
            }
            0x1b => {
                // Path Response
                // https://datatracker.ietf.org/doc/html/rfc9000#name-path_response-frames
                let data = data.read_u64::<NetworkEndian>()?;

                Frame::PathResponse(PathResponse { data })
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
                let reason_phrase = data.slice(reason_phrase_length.into())?;

                Frame::ConnectionClose(ConnectionClose {
                    err_code,
                    frame_type,
                    reason_phrase,
                })
            }
            0x1e => {
                // Handshake Done
                // https://datatracker.ietf.org/doc/html/rfc9000#name-handshake_done-frames
                Frame::HandshakeDone
            }
            _ => Err("Unknown frame type")?,
        };

        Ok((frame, data))
    }
}
