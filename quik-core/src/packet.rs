use crate::common::ConnectionId;
// Packets handled by the middle layer

pub enum Packet<'a> {
    VersionNegotiation(VersionNegotiation<'a>),
    Initial(Initial<'a>),
    ZeroRTT(ZeroRTT),
    Handshake(Handshake),
    Retry(Retry<'a>),
    OneRtt(OneRtt),
}

pub struct VersionNegotiation<'a> {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub supported_versions: &'a [u32],
}

pub struct Initial<'a> {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    pub token: &'a [u8],
    // variable length
    pub packet_number: u32,
}

pub struct ZeroRTT {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    // variable length
    pub packet_number: u32,
}

pub struct Handshake {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    // variable length
    pub packet_number: u32,
}

pub struct Retry<'a> {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    pub retry_token: &'a [u8],
    pub retry_integrity_tag: u128,
}

// This one actually uses the short header
pub struct OneRtt {
    pub dst_cid: ConnectionId,
    pub spin: u8,
    pub key_phase: u8,

    // variable length
    pub packet_number: u32,
}
