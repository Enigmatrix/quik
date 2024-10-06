use crate::common::{ConnectionId, StreamId, VarInt};

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
    HandshakeDone
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
    pub ack_range: Vec<AckRange>,
    pub ecn_counts: Vec<EcnCounts>,
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
