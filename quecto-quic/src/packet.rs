use crate::common::ConnectionId;
use quecto_util::*;
// Packets handled by the middle layer

pub struct VersionNegotiationPacket<'a> {
    pub src_conn_id: ConnectionId,
    pub dest_conn_id: ConnectionId,
    pub supported_versions: &'a [u32],
}

pub struct InitialPacket<'a> {
    pub src_conn_id: ConnectionId,
    pub dest_conn_id: ConnectionId,
    pub version: u32,

    pub token: &'a [u8],
    // variable length
    pub packet_number: u32,
}

pub struct ZeroRTTPacket {
    pub src_conn_id: ConnectionId,
    pub dest_conn_id: ConnectionId,
    pub version: u32,

    // variable length
    pub packet_number: u32,
}

pub struct HandshakePacket {
    pub src_conn_id: ConnectionId,
    pub dest_conn_id: ConnectionId,
    pub version: u32,

    // variable length
    pub packet_number: u32,
}

pub struct RetryPacket<'a> {
    pub src_conn_id: ConnectionId,
    pub dest_conn_id: ConnectionId,
    pub version: u32,

    pub retry_token: &'a [u8],
    pub retry_integrity_tag: u128,
}

// This one actually uses the short header
pub struct OneRttPacket {
    pub dest_conn_id: ConnectionId,
    pub spin: u8,
    pub key_phase: u8,

    // variable length
    pub packet_number: u32,
}
