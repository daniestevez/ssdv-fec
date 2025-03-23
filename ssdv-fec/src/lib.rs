//! # SSDV systematic erasure FEC
//!
//! This crate implements a systematic erasure FEC scheme for
//! [SSDV](https://github.com/fsphil/ssdv). The FEC scheme is based on a
//! Reed-Solomon code over GF(2¹⁶) used as a fountain-like code. This idea was
//! first described in the blog post
//! [An erasure FEC for SSDV](https://destevez.net/2023/05/an-erasure-fec-for-ssdv/)
//! by the author of this crate.
//!
//! Given an SSDV image formed by k SSDV packets, the FEC encoder can generate
//! up to 2¹⁶ different SSDV packets identified by a packet ID from 0 to
//! 2¹⁶-1. The packets with IDs from 0 to k-1 are called "systematic
//! packets" and are the same as the k packets of the original image. The
//! remaining packets are called "FEC packets". Each packet can be generated on
//! demand according to the needs of the transmitter. The large amount of 2¹⁶
//! distinct packets than can be generated provides a virtually limitless source
//! of packets. The receiver can recover the original SSDV image from any set of
//! k distinct packets.
//!
//! Several SSDV packet formats are supported:
//!
//! - The no-FEC
//!   [standard packet format](https://ukhas.org.uk/doku.php?id=guides:ssdv#packet_format)
//!   implemented by the
//!   [upstream SSDV](https://github.com/fsphil/ssdv). This is a 256-byte packet
//!   format that includes a callsign.
//!
//! - The custom packet format used by Longjiang-2, which is implemented in a
//!   [fork of SSVD](https://github.com/daniestevez/ssdv). This is a 218-byte
//!   packet format that omits the sync byte, packet type and callsign fields (but
//!   includes them implicitly in the generation of the CRC-32).
//!
//! Other packet formats can be supported by implementing the [`SSDVParameters`]
//! or the [`SSDVPacket`] trait.
//!
//! This implementation of the FEC scheme uses 218-byte SSDV packets following
//! the format used by Longjiang-2, which omits the sync byte, packet type and
//! callsign fields (but includes them implicitly in the generation of the
//! CRC-32).
//!
//! The crate supports `no_std` and the implementation is designed with small
//! microcontrollers in mind. The GF(2¹⁶) arithmetic only uses two tables of 256
//! bytes each that are included in the `.rodata` section. The FEC encoder and
//! decoder work with externally provided slices, giving freedom as to how to
//! perform memory allocation, and do the computations in-place when
//! possible. The memory required for encoding corresponds to a buffer
//! containing the k SSDV packets of the original image, and a buffer containing
//! the packet being encoded. The memory required for decoding corresponds to a
//! buffer containing at least k distinct received SSDV packets, and another
//! buffer where the k SSDV packets that compose the original image can be
//! written. Besides these buffers, the algorithms use only a small amount of
//! stack space.
//!
//! A simple CLI application that can perform encoding and decoding can be built
//! with the `cli` feature, which is enabled by default.

#![warn(missing_docs)]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

#[cfg(feature = "cli")]
pub mod cli;

mod crc;
mod fec;
pub use fec::{Decoder, DecoderError, Encoder, EncoderError};
mod gf64k;
pub use gf64k::{GF64K, GF256};
mod ssdv;
pub use ssdv::{SSDVPacket, SSDVPacketArray, SSDVParameters};

pub mod packet_formats;

#[cfg(test)]
mod test_data;
