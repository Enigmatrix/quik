use crate::common::ConnectionId;
use quik_util::*;
// Packets handled by the middle layer

pub struct VersionNegotiationPacket<'a> {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub supported_versions: &'a [u32],
}

pub struct InitialPacket<'a> {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    pub token: &'a [u8],
    // variable length
    pub packet_number: u32,
}

pub struct ZeroRTTPacket {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    // variable length
    pub packet_number: u32,
}

pub struct HandshakePacket {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    // variable length
    pub packet_number: u32,
}

pub struct RetryPacket<'a> {
    pub src_cid: ConnectionId,
    pub dst_cid: ConnectionId,
    pub version: u32,

    pub retry_token: &'a [u8],
    pub retry_integrity_tag: u128,
}

// This one actually uses the short header
pub struct OneRttPacket {
    pub dst_cid: ConnectionId,
    pub spin: u8,
    pub key_phase: u8,

    // variable length
    pub packet_number: u32,
}
