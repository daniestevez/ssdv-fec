//! SSDV packet formats.
//!
//! This module contains submodules that define each of the SSDV packet formats
//! supported by this crate.

use crate::{SSDVPacketArray, SSDVParameters};
use core::borrow::Borrow;
use generic_array::{ArrayLength, GenericArray, typenum};

/// No-FEC standard SSDV packet format.
///
/// This module contains the packet format definition for the
/// [no-FEC standard SSDV packet format](https://ukhas.org.uk/doku.php?id=guides:ssdv#packet_format).
pub mod no_fec {
    use super::*;

    /// No-FEC standard SSDV packet format parameters.
    ///
    /// This ZST implements [`SSDVParameters`] to define the parameters of the
    /// no-FEC standard SSDV packet format.
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
    pub struct Parameters {}

    impl SSDVParameters for Parameters {
        type PacketLen = typenum::U256;

        type ArrayType = <Self::PacketLen as ArrayLength>::ArrayType<u8>;

        type DataLen = typenum::U240;

        type CrcDataLen = typenum::U251;

        type CallsignLen = typenum::U4;

        const CRC_DATA_OFFSET: usize = 1;

        const CALLSIGN_OFFSET: usize = 2;

        const IMAGE_ID_OFFSET: usize = 6;

        fn set_fixed_fields(packet: &mut GenericArray<u8, Self::PacketLen>) {
            packet[0] = 0x55; // sync byte
            packet[1] = 0x67; // packet type: no-FEC mode
        }
    }

    /// No-FEC standard SSDV packet.
    ///
    /// This is the [`SSDVPacketArray`] corresponding to the no-FEC standard
    /// SSDV packet format.
    pub type SSDVPacket = SSDVPacketArray<Parameters>;
}

/// Longjiang-2 SSDV packet format.
///
/// This module contains the packet format definition for the custom SSDV format
/// used during the Longjiang-2 mission. This is a 218-byte packet format that
/// omits the sync byte, packet type and callsign fields (but includes them
/// implicitly in the generation of the CRC-32).
pub mod longjiang2 {
    use super::*;

    /// Longjiang-2 SSDV packet format parameters.
    ///
    /// This ZST implements [`SSDVParameters`] to define the parameters of the
    /// Longjiang-2 SSDV packet format.
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
    pub struct Parameters {}

    impl SSDVParameters for Parameters {
        type PacketLen = typenum::U218;

        type ArrayType = <Self::PacketLen as ArrayLength>::ArrayType<u8>;

        type DataLen = typenum::U208;

        type CrcDataLen = typenum::U214;

        type CallsignLen = typenum::U0;

        const CRC_DATA_OFFSET: usize = 0;

        const CALLSIGN_OFFSET: usize = 0;

        const IMAGE_ID_OFFSET: usize = 0;

        fn compute_crc32<I, T>(data: I) -> u32
        where
            I: Iterator<Item = T>,
            T: Borrow<u8>,
        {
            crate::crc::crc32_dslwp(data)
        }
    }

    /// Longjiang-2 SSDV packet.
    ///
    /// This is the [`SSDVPacketArray`] corresponding to the custom Longjiang-2
    /// packet format.
    pub type SSDVPacket = SSDVPacketArray<Parameters>;
}
