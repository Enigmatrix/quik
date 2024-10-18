use std::io;

use quik_util::*;

use crate::crypto::Crypto;
use crate::wire::{ConnectionId, PacketNumber, VarInt};
// Packets handled by the middle layer

pub enum Packet<'a> {
  VersionNegotiation(VersionNegotiation),
  Initial(Initial<'a>),
  ZeroRTT(ZeroRTT),
  Handshake(Handshake),
  Retry(Retry<'a>),
  OneRtt(OneRtt),
}

impl Packet<'_> {
  pub fn dst_cid(&self) -> &ConnectionId {
    match self {
      Packet::VersionNegotiation(vn) => &vn.dst_cid,
      Packet::Initial(i) => &i.dst_cid,
      Packet::ZeroRTT(z) => &z.dst_cid,
      Packet::Handshake(h) => &h.dst_cid,
      Packet::Retry(r) => &r.dst_cid,
      Packet::OneRtt(o) => &o.dst_cid,
    }
  }
}

pub struct VersionNegotiation {
  pub src_cid: ConnectionId,
  pub dst_cid: ConnectionId,
  // TODO ew this isn't zero copy
  pub supported_versions: Vec<u32>,
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

pub enum RemainingBuf {
  Decrypted(Vec<u8>),
  // Raw(&'a [u8]),
  None,
}

impl<'a> Packet<'a> {
  pub async fn parse(
    crypto: &impl Crypto,
    mut data: &'a [u8],
  ) -> Result<(Packet<'a>, RemainingBuf)> {
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

        let packet = Packet::VersionNegotiation(VersionNegotiation {
          src_cid,
          dst_cid: dst_cid.clone(),
          supported_versions: versions,
        });
        return Ok((packet, RemainingBuf::None));
      }
      match packet_type {
        0b00 => {
          // Initial packet
          // https://datatracker.ietf.org/doc/html/rfc9000#name-initial-packet

          let token_length = VarInt::parse(&mut data)?;
          let token = data.slice(token_length.into())?;
          let length = VarInt::parse(&mut data)?; // TODO use this
          let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;

          let packet = Packet::Initial(Initial {
            src_cid,
            dst_cid: dst_cid.clone(),
            version,
            token,
            packet_number,
          });
          let payload = crypto
            .decrypt_initial_data(dst_cid, version, false, &mut data)
            .await?;
          Ok((packet, RemainingBuf::Decrypted(payload)))
        }
        0b01 => {
          // 0-RTT packet
          // https://datatracker.ietf.org/doc/html/rfc9000#name-0-rtt

          let length = VarInt::parse(&mut data)?; // TODO use this
          let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
          let payload = data.to_vec(); // TODO: decrypt

          let packet = Packet::ZeroRTT(ZeroRTT {
            src_cid,
            dst_cid: dst_cid.clone(),
            version,
            packet_number,
          });
          Ok((packet, RemainingBuf::Decrypted(payload)))
        }
        0b10 => {
          // Handshake packet
          // https://datatracker.ietf.org/doc/html/rfc9000#packet-handshake

          let length = VarInt::parse(&mut data)?; // TODO use this
          let packet_number = PacketNumber::parse(&mut data, packet_number_length)?;
          let payload = data.to_vec(); // TODO: decrypt

          let packet = Packet::Handshake(Handshake {
            src_cid,
            dst_cid: dst_cid.clone(),
            version,
            packet_number,
          });
          Ok((packet, RemainingBuf::Decrypted(payload)))
        }
        0b11 => {
          // Retry packet
          // https://datatracker.ietf.org/doc/html/rfc9000#name-retry-packet

          // TODO is this encoding even correct? wtf is a retry token?
          let (retry_token, retry_integrity_tag) = data
            .split_last_chunk::<16>() // 128bits/8 = 16 bytes
            .ok_or_else(|| "Packet too short for Retry Token")?;
          let retry_integrity_tag = (&retry_integrity_tag[..]).read_u128::<NetworkEndian>()?;

          let packet = Packet::Retry(Retry {
            src_cid,
            dst_cid: dst_cid.clone(),
            version,
            retry_token,
            retry_integrity_tag,
          });
          Ok((packet, RemainingBuf::None))
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
      let payload = data.to_vec(); // TODO: decrypt

      let packet = Packet::OneRtt(OneRtt {
        dst_cid: dst_cid.clone(),
        packet_number,
        spin,
        key_phase,
      });
      Ok((packet, RemainingBuf::Decrypted(payload)))
    }
  }
}
