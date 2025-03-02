use core::borrow::Borrow;
use generic_array::{typenum::Unsigned, ArrayLength, GenericArray};

/// SSDV packet.
///
/// This trait represents an abstract SSDV packet. Implementors of this trait
/// correspond to the different SSDV packet formats that exist (for instance,
/// the standard no-FEC format, or the custom format used for Lonjiang-2).
pub trait SSDVPacket: Default + Copy {
    /// Length of the data field of an SSDV packet.
    ///
    /// The length is specified in bytes as an unsigned integer type from
    /// [`generic_array::typenum`].
    ///
    /// The data field includes the MCU offset, MCU index and payload fields of
    /// the SSDV packet.
    type DataLen: ArrayLength;

    /// Length of the data taken into account for CRC-32 calculation.
    ///
    /// The length is specified in bytes as an unsigned integer type from
    /// [`generic_array::typenum`].
    type CrcDataLen: ArrayLength;

    /// Length of the callsign field.
    ///
    /// The length is specified in bytes as an unsigned integer type from
    /// [`generic_array::typenum`].
    type CallsignLen: ArrayLength;

    /// Sets the fields of the packet that have fixed values.
    ///
    /// This function writes into this packet the fields that have a fixed
    /// value. For instance, for the standard SSDV format, this is the sync byte
    /// and the packet type field.
    fn set_fixed_fields(&mut self);

    /// Returns a reference to an array containing the callsign field.
    fn callsign(&self) -> &GenericArray<u8, Self::CallsignLen>;

    /// Returns a mutable reference to an array containing the callsign field.
    fn callsign_as_mut(&mut self) -> &mut GenericArray<u8, Self::CallsignLen>;

    /// Returns the value of the image ID field.
    fn image_id(&self) -> u8;

    /// Sets the value of the image ID field.
    fn set_image_id(&mut self, image_id: u8);

    /// Returns the value of the packet ID field.
    fn packet_id(&self) -> u16;

    /// Sets the value of the packet ID field.
    fn set_packet_id(&mut self, packet_id: u16);

    /// Returns the value of the width field.
    ///
    /// The width field is only present in systematic packets. If this function
    /// is called on a FEC packet it returns `None`.
    fn width(&self) -> Option<u8>;

    /// Sets the value of the width field.
    ///
    /// The width field is only present in systematic packets. This function
    /// should only be called for systematic packets.
    fn set_width(&mut self, width: u8);

    /// Returns the value of the height field.
    ///
    /// The height field is only present in systematic packets. If this function
    /// is called on a FEC packet it returns `None`.
    fn height(&self) -> Option<u8>;

    /// Sets the value of the height field.
    ///
    /// The height field is only present in systematic packets. This function
    /// should only be called for systematic packets.
    fn set_height(&mut self, height: u8);

    /// Returns the value of the number of systematic packets field.
    ///
    /// This field is only present in FEC packets. If this function is called on
    /// a systematic packet it returns `None`.
    fn number_systematic_packets(&self) -> Option<u16>;

    /// Sets the value of the number of systematic packets field.
    ///
    /// This field is only present in FEC packets. This function
    /// should only be called for FEC packets.
    fn set_number_systematic_packets(&mut self, number_systematic_packets: u16);

    /// Returns the value of the flags field.
    fn flags(&self) -> u8;

    /// Sets the value of the flags field.
    fn set_flags(&mut self, flags: u8);

    /// Returns true if the packet has the EOI flag set.
    fn is_eoi(&self) -> bool {
        self.flags() & 0x4 != 0
    }

    /// Sets the value of the EOI flag.
    fn set_eoi(&mut self, eoi: bool) {
        self.set_flags((self.flags() & !0x4) | (u8::from(eoi) << 2));
    }

    /// Returns true if the packet has the FEC packet flag set.
    fn is_fec_packet(&self) -> bool {
        self.flags() & 0x40 != 0
    }

    /// Sets the value of the FEC packet flag.
    fn set_fec_packet(&mut self, fec_packet: bool) {
        self.set_flags((self.flags() & !0x40) | (u8::from(fec_packet) << 6));
    }

    /// Returns a reference to an array containing the packet data field.
    ///
    /// The data field includes the MCU offset, MCU index and payload fields of
    /// the SSDV packet.
    fn data(&self) -> &GenericArray<u8, Self::DataLen>;

    /// Returns a mutable reference to an array containing the packet data field.
    ///
    /// The data field includes the MCU offset, MCU index and payload fields of
    /// the SSDV packet.
    fn data_as_mut(&mut self) -> &mut GenericArray<u8, Self::DataLen>;

    /// Returns a reference to an array containing the part of the packet
    /// covered by the CRC-32 calculation.
    fn crc32_data(&self) -> &GenericArray<u8, Self::CrcDataLen>;

    /// Computes the CRC-32 of some data with the CRC-32 algorithm used by this
    /// SSDV packet format.
    ///
    /// The default implementation corresponds to the standard CRC-32 algorithm.
    fn compute_crc32<I, T>(data: I) -> u32
    where
        I: Iterator<Item = T>,
        T: Borrow<u8>,
    {
        crate::crc::crc32(data)
    }

    /// Calculates the CRC-32 of the data in the packet.
    ///
    /// This function returns the CRC-32 of the array returned by
    /// [`Self::crc32_data`].
    fn computed_crc32(&self) -> u32 {
        Self::compute_crc32(self.crc32_data().iter())
    }

    /// Returns the value of the CRC-32 field of the packet.
    fn crc32(&self) -> u32;

    /// Sets the value of the CRC-32 field of the packet.
    fn set_crc32(&mut self, crc32: u32);

    /// Returns `true` if the CRC-32 of the packet is correct.
    fn crc32_is_valid(&self) -> bool {
        self.computed_crc32() == self.crc32()
    }

    /// Sets the CRC-32 field of the packet to the CRC computed from the data.
    ///
    /// This function modifies the CRC-32 field of the packet by setting it to
    /// the CRC-32 returned by [`Self::crc32`].
    fn update_crc32(&mut self) {
        self.set_crc32(self.computed_crc32());
    }
}

/// SSDV format parameters.
///
/// This trait describes an SSDV packet format in a minimalistic way, and allows
/// the usage of the [`SSDVPacketArray`] struct to store SSDV packets of this
/// format in a [`GenericArray`]. Implementing this trait is the usual way of
/// adding support for an SSDV packet format.
///
/// Implementing an SSDV packet format using this trait has the following
/// flexibility:
///
/// - The image ID, packet ID, width, height, flags, and data (which includes
///   MCU offset, MCU index, and payload) fields must be adjacent and have the
///   same lengths as defined in the
///   [standard SSDV packet format](https://ukhas.org.uk/doku.php?id=guides:ssdv#packet_format).
///   The offset of the image ID field within the packet can be arbitrary (allowing
///   for any preceding fields in the format), and is defined with
///   [`SSDVParameters::IMAGE_ID_OFFSET`].
///
/// - The callsign field is optional and can have an arbitrary length ad
///   position within the packet. These are defined with
///   [`SSDVParameters::CALLSIGN_OFFSET`] and [`SSDVParameters::CallsignLen`].
///
/// - The part of the packet that is covered by the CRC-32 calculation can be
///   defined arbitrarily using [`SSDVParameters::CRC_DATA_OFFSET`] and
///   [`SSDVParameters::CrcDataLen`], but it needs to be a contiguous segment.
///   The CRC-32 is placed at the end of the packet. A custom CRC-32 algorithm
///   can be defined by overriding [`SSDVParameters::compute_crc32`].
///
/// - Arbitrary packet fields with fixed values are supported with
///   [`SSDVParameters::set_fixed_fields`].
pub trait SSDVParameters {
    /// Length of the SSDV packet.
    ///
    /// The length is specified in bytes as an unsigned integer type from
    /// [`generic_array::typenum`].
    type PacketLen: ArrayLength<ArrayType<u8> = Self::ArrayType>;

    /// Array type.
    ///
    /// This type exists only for technical reasons. It is needed so that the
    /// Rust compiler can infer that [`GenericArray<u8, Self::PacketLen>`]
    /// implements [`Copy`]. Typically, this type should be set to
    /// `<Self::PacketLen as ArrayLength>::ArrayType<u8>`.
    type ArrayType: Copy;

    /// Length of the data field of an SSDV packet.
    ///
    /// The length is specified in bytes as an unsigned integer type from
    /// [`generic_array::typenum`].
    type DataLen: ArrayLength;

    /// Length of the data that is considered in the CRC-32 calculation.
    ///
    /// The length is specified in bytes as an unsigned integer type from
    /// [`generic_array::typenum`].
    type CrcDataLen: ArrayLength;

    /// Length of the callsign field.
    ///
    /// The length is specified in bytes as an unsigned integer type from
    /// [`generic_array::typenum`]. The length should be
    /// [`generic_array::typenum::U0`] for formats which do not contain a
    /// callsign field.
    type CallsignLen: ArrayLength;

    /// Offset of the data that is considered in the CRC-32 calculation.
    ///
    /// This gives the offset in bytes, with respect to the beginning of the
    /// packet, of the data that is considered in the CRC-32 calculation.
    const CRC_DATA_OFFSET: usize;

    /// Offset of the callsign field.
    ///
    /// This gives the offset in bytes, with respect to the beginning of the
    /// packet, of the callsign field. Formats which do not contain a callsign
    /// field should set this constant to `0`.
    const CALLSIGN_OFFSET: usize;

    /// Offset of the image ID field.
    ///
    /// This gives the offset in bytes, with respect to the beginning of the
    /// packet, of the image ID field.
    const IMAGE_ID_OFFSET: usize;

    /// Computes the CRC-32 of some data.
    ///
    /// This function returns the CRC-32 corresponding to `data`, using the
    /// CRC-32 algorithm specified by this packet format.
    ///
    /// The default implementation uses a standard CRC-32 algorithm.
    fn compute_crc32<I, T>(data: I) -> u32
    where
        I: Iterator<Item = T>,
        T: Borrow<u8>,
    {
        crate::crc::crc32(data)
    }

    /// Sets the fields of the packet that have fixed values.
    ///
    /// This function writes into the [`GenericArray`] containing this packet
    /// the fields that have a fixed value. For instance, for the standard SSDV
    /// format, this is the sync byte and the packet type field.
    ///
    /// The default implementation does nothing.
    fn set_fixed_fields(_packet: &mut GenericArray<u8, Self::PacketLen>) {}
}

/// SSDV packet stored in a [`GenericArray`].
///
/// This struct stores an SSDV packet in a [`GenericArray`] transparently (so
/// the packet has the same ABI as an array `[u8; N]`). The struct implements
/// the [`SSDVPacket`] trait by using the parameters defined in the
/// [`SSDVParameters`] implementation of the type parameter `P`.
#[repr(transparent)]
pub struct SSDVPacketArray<P: SSDVParameters>(pub GenericArray<u8, P::PacketLen>);

impl<P: SSDVParameters> SSDVPacketArray<P> {
    /// Creates a new SSDV packet from a slice.
    ///
    /// This function creates a new SSDV packet by copying the data in
    /// `slice`. If the length of `slice` is different from the SSDV packet
    /// length specified by `P::PacketLen`, an error is returned.
    pub fn new_from_slice(slice: &[u8]) -> Result<Self, generic_array::LengthError> {
        let array: &GenericArray<u8, P::PacketLen> = slice.try_into()?;
        Ok(Self(*array))
    }
}

impl<P: SSDVParameters> core::fmt::Debug for SSDVPacketArray<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SSDVPacketArray")
            .field("0", &self.0)
            .finish()
    }
}

impl<P: SSDVParameters> Clone for SSDVPacketArray<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: SSDVParameters> Copy for SSDVPacketArray<P> {}

impl<P: SSDVParameters> PartialEq for SSDVPacketArray<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P: SSDVParameters> Eq for SSDVPacketArray<P> {}

impl<P: SSDVParameters> core::hash::Hash for SSDVPacketArray<P> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<P: SSDVParameters> Default for SSDVPacketArray<P> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<P: SSDVParameters> From<GenericArray<u8, P::PacketLen>> for SSDVPacketArray<P> {
    fn from(value: GenericArray<u8, P::PacketLen>) -> Self {
        Self(value)
    }
}

impl<P: SSDVParameters> From<SSDVPacketArray<P>> for GenericArray<u8, P::PacketLen> {
    fn from(value: SSDVPacketArray<P>) -> Self {
        value.0
    }
}

impl<P: SSDVParameters> SSDVPacket for SSDVPacketArray<P>
where
    <P::PacketLen as ArrayLength>::ArrayType<u8>: Copy,
{
    type DataLen = P::DataLen;

    type CrcDataLen = P::CrcDataLen;

    type CallsignLen = P::CallsignLen;

    fn set_fixed_fields(&mut self) {
        P::set_fixed_fields(&mut self.0);
    }

    fn callsign(&self) -> &GenericArray<u8, Self::CallsignLen> {
        self.0[P::CALLSIGN_OFFSET..P::CALLSIGN_OFFSET + Self::CallsignLen::USIZE]
            .try_into()
            .unwrap()
    }

    fn callsign_as_mut(&mut self) -> &mut GenericArray<u8, Self::CallsignLen> {
        (&mut self.0[P::CALLSIGN_OFFSET..P::CALLSIGN_OFFSET + Self::CallsignLen::USIZE])
            .try_into()
            .unwrap()
    }

    fn image_id(&self) -> u8 {
        self.0[P::IMAGE_ID_OFFSET]
    }

    fn set_image_id(&mut self, image_id: u8) {
        self.0[P::IMAGE_ID_OFFSET] = image_id;
    }

    fn packet_id(&self) -> u16 {
        u16::from_be_bytes(
            self.0[P::IMAGE_ID_OFFSET + 1..P::IMAGE_ID_OFFSET + 3]
                .try_into()
                .unwrap(),
        )
    }

    fn set_packet_id(&mut self, packet_id: u16) {
        self.0[P::IMAGE_ID_OFFSET + 1] = (packet_id >> 8) as u8;
        self.0[P::IMAGE_ID_OFFSET + 2] = (packet_id & 0xff) as u8;
    }

    fn width(&self) -> Option<u8> {
        if self.is_fec_packet() {
            None
        } else {
            Some(self.0[P::IMAGE_ID_OFFSET + 3])
        }
    }

    fn set_width(&mut self, width: u8) {
        self.0[P::IMAGE_ID_OFFSET + 3] = width;
    }

    fn height(&self) -> Option<u8> {
        if self.is_fec_packet() {
            None
        } else {
            Some(self.0[P::IMAGE_ID_OFFSET + 4])
        }
    }

    fn set_height(&mut self, height: u8) {
        self.0[P::IMAGE_ID_OFFSET + 4] = height;
    }

    fn number_systematic_packets(&self) -> Option<u16> {
        if self.is_fec_packet() {
            Some(u16::from_be_bytes(
                self.0[P::IMAGE_ID_OFFSET + 3..P::IMAGE_ID_OFFSET + 5]
                    .try_into()
                    .unwrap(),
            ))
        } else {
            None
        }
    }

    fn set_number_systematic_packets(&mut self, number_systematic_packets: u16) {
        self.0[P::IMAGE_ID_OFFSET + 3] = (number_systematic_packets >> 8) as u8;
        self.0[P::IMAGE_ID_OFFSET + 4] = (number_systematic_packets & 0xff) as u8;
    }

    fn flags(&self) -> u8 {
        self.0[P::IMAGE_ID_OFFSET + 5]
    }

    fn set_flags(&mut self, flags: u8) {
        self.0[P::IMAGE_ID_OFFSET + 5] = flags;
    }

    fn data(&self) -> &GenericArray<u8, Self::DataLen> {
        self.0[P::IMAGE_ID_OFFSET + 6..P::IMAGE_ID_OFFSET + 6 + Self::DataLen::USIZE]
            .try_into()
            .unwrap()
    }

    fn data_as_mut(&mut self) -> &mut GenericArray<u8, Self::DataLen> {
        (&mut self.0[P::IMAGE_ID_OFFSET + 6..P::IMAGE_ID_OFFSET + 6 + Self::DataLen::USIZE])
            .try_into()
            .unwrap()
    }

    fn crc32_data(&self) -> &GenericArray<u8, Self::CrcDataLen> {
        self.0[P::CRC_DATA_OFFSET..P::CRC_DATA_OFFSET + P::CrcDataLen::USIZE]
            .try_into()
            .unwrap()
    }

    fn compute_crc32<I, T>(data: I) -> u32
    where
        I: Iterator<Item = T>,
        T: Borrow<u8>,
    {
        P::compute_crc32(data)
    }

    fn crc32(&self) -> u32 {
        u32::from_be_bytes(self.0[P::PacketLen::USIZE - 4..].try_into().unwrap())
    }

    fn set_crc32(&mut self, crc32: u32) {
        self.0[P::PacketLen::USIZE - 4..].copy_from_slice(&crc32.to_be_bytes());
    }
}
