/// SSDV packet.
///
/// This struct wraps an array containing an SSDV packet and provides some
/// convenience methods for accessing the fields of the packet.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SSDVPacket(pub [u8; SSDV_PACKET_LEN]);

/// Length of an SSDV packet in bytes.
pub const SSDV_PACKET_LEN: usize = 218;

/// Length of the data field of an SSDV packet.
pub const SSDV_DATA_LEN: usize = 208;

impl SSDVPacket {
    /// Returns a new SSDVPacket full of zeros.
    pub fn zeroed() -> SSDVPacket {
        SSDVPacket([0u8; SSDV_PACKET_LEN])
    }

    /// Returns the value of the image ID field.
    pub fn image_id(&self) -> u8 {
        self.0[0]
    }

    /// Sets the value of the image ID field.
    pub fn set_image_id(&mut self, image_id: u8) {
        self.0[0] = image_id;
    }

    /// Returns the value of the packet ID field.
    pub fn packet_id(&self) -> u16 {
        u16::from_be_bytes(self.0[1..3].try_into().unwrap())
    }

    /// Sets the value of the packet ID field.
    pub fn set_packet_id(&mut self, packet_id: u16) {
        self.0[1] = (packet_id >> 8) as u8;
        self.0[2] = (packet_id & 0xff) as u8;
    }

    /// Returns the value of the width field.
    ///
    /// The width field is only present in systematic packets. If this function
    /// is called on a FEC packet it returns `None`.
    pub fn width(&self) -> Option<u8> {
        if self.is_fec_packet() {
            None
        } else {
            Some(self.0[3])
        }
    }

    /// Sets the value of the width field.
    ///
    /// The width field is only present in systematic packets. This function
    /// should only be called for systematic packets.
    pub fn set_width(&mut self, width: u8) {
        self.0[3] = width;
    }

    /// Returns the value of the height field.
    ///
    /// The height field is only present in systematic packets. If this function
    /// is called on a FEC packet it returns `None`.
    pub fn height(&self) -> Option<u8> {
        if self.is_fec_packet() {
            None
        } else {
            Some(self.0[4])
        }
    }

    /// Sets the value of the height field.
    ///
    /// The height field is only present in systematic packets. This function
    /// should only be called for systematic packets.
    pub fn set_height(&mut self, height: u8) {
        self.0[4] = height;
    }

    /// Returns the value of the number of system packets field.
    ///
    /// This field is only present in FEC packets. If this function is called on
    /// a systematic packet it returns `None`.
    pub fn number_systematic_packets(&self) -> Option<u16> {
        if self.is_fec_packet() {
            Some(u16::from_be_bytes(self.0[3..5].try_into().unwrap()))
        } else {
            None
        }
    }

    /// Sets the value of the number of systematic packets field.
    ///
    /// This field is only present in FEC packets. This function
    /// should only be called for FEC packets.
    pub fn set_number_systematic_packets(&mut self, number_systematic_packets: u16) {
        self.0[3] = (number_systematic_packets >> 8) as u8;
        self.0[4] = (number_systematic_packets & 0xff) as u8;
    }

    /// Returns the value of the flags field.
    pub fn flags(&self) -> u8 {
        self.0[5]
    }

    /// Sets the value of the flags field.
    pub fn set_flags(&mut self, flags: u8) {
        self.0[5] = flags;
    }

    /// Returns true if the packet has the EOI flag set.
    pub fn is_eoi(&self) -> bool {
        self.flags() & 0x4 != 0
    }

    /// Sets the value of the EOI flag.
    pub fn set_eoi(&mut self, eoi: bool) {
        self.0[5] = (self.0[5] & !0x4) | (u8::from(eoi) << 2);
    }

    /// Returns true if the packet has the FEC packet flag set.
    pub fn is_fec_packet(&self) -> bool {
        self.flags() & 0x40 != 0
    }

    /// Sets the value of the FEC packet flag.
    pub fn set_fec_packet(&mut self, fec_packet: bool) {
        self.0[5] = (self.0[5] & !0x40) | (u8::from(fec_packet) << 6);
    }

    /// Returns a reference to the sub-array that contains the packet data.
    pub fn data(&self) -> &[u8; SSDV_DATA_LEN] {
        self.0[6..6 + SSDV_DATA_LEN].try_into().unwrap()
    }

    /// Returns a reference to the sub-array that contains the packet data.
    pub fn data_as_mut(&mut self) -> &mut [u8; SSDV_DATA_LEN] {
        (&mut self.0[6..6 + SSDV_DATA_LEN]).try_into().unwrap()
    }

    /// Returns a reference to the sub-array covered by the CRC-32 calculation.
    pub fn crc32_data(&self) -> &[u8; SSDV_PACKET_LEN - 4] {
        self.0[..SSDV_PACKET_LEN - 4].try_into().unwrap()
    }

    /// Returns the value of the CRC-32 field of the packet.
    pub fn crc32(&self) -> u32 {
        u32::from_be_bytes(self.0[SSDV_PACKET_LEN - 4..].try_into().unwrap())
    }

    /// Sets the value of the CRC-32 field of the packet.
    pub fn set_crc32(&mut self, crc32: u32) {
        self.0[SSDV_PACKET_LEN - 4..].copy_from_slice(&crc32.to_be_bytes());
    }
}
