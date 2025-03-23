use crate::{GF64K, SSDVPacket};
use generic_array::GenericArray;
#[cfg(feature = "std")]
use thiserror::Error;

/// SSDV FEC encoder.
///
/// This struct is used to encode an arbitrary number of packets for an SSDV
/// image in a fountain-code-like manner. The encoder is initialized with
/// [`Encoder::new`] by giving it the SSDV image packets. Afterwards, the
/// [`Encoder::encode`] function can be called to generate a packet with an
/// arbitrary `packet_id`.
///
/// The struct contains a mutable reference to a slice containing the SSDV
/// packets of the image. The lifetime of this slice is given by the lifetime
/// parameter `'a`.
#[derive(Debug)]
pub struct Encoder<'a, S> {
    buffer: &'a mut [S],
}

/// Error produced by the SSDV FEC encoder.
///
/// This enum lists the errors that can be produced by [`Encoder`].
#[allow(clippy::enum_variant_names)] // this is triggered because all the variants end in Input
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum EncoderError {
    /// The encoder input is empty.
    #[cfg_attr(feature = "std", error("encoder input is empty"))]
    EmptyInput,
    /// The encoder input is too long.
    #[cfg_attr(feature = "std", error("encoder input is too long"))]
    TooLongInput,
    /// There is a non-systematic packet in the encoder input.
    #[cfg_attr(feature = "std", error("non-systematic packet in encoder input"))]
    NonSystematicInput,
}

// Computes
// w_j^{-1} = \prod_{m \neq j} (x_j - x_m).
fn wj_inv(j: u16, k: u16) -> GF64K {
    let xj = GF64K::from(j);
    let mut ret = GF64K::from(1);
    for m in 0..k {
        if m != j {
            let xm = GF64K::from(m);
            ret *= xj - xm;
        }
    }
    ret
}

impl<S: SSDVPacket> Encoder<'_, S> {
    /// Creates a new FEC encoder for an SSDV image.
    ///
    /// The systematic packets for the image are given in the slice
    /// `systematic_packets`. They must be in order and without repetitions. The
    /// encoder works in-place in this slice, modifying its contents.
    ///
    /// If there is a problem with the input contents, this function returns an
    /// error. Otherwise, an [`Encoder`] struct on which
    /// [`encode`](`Encoder::encode`) can be called is returned.
    pub fn new(systematic_packets: &mut [S]) -> Result<Encoder<S>, EncoderError> {
        if systematic_packets.is_empty() {
            return Err(EncoderError::EmptyInput);
        }
        if systematic_packets.len() > usize::from(u16::MAX) {
            return Err(EncoderError::TooLongInput);
        }
        // only check the first packet for efficiency
        if systematic_packets[0].is_fec_packet() {
            return Err(EncoderError::NonSystematicInput);
        }
        let mut encoder = Encoder {
            buffer: systematic_packets,
        };
        encoder.values_to_lagrange();
        Ok(encoder)
    }

    fn values_to_lagrange(&mut self) {
        // The Lagrange polynomial L(x) that interpolates
        // L(x_j) = y_j
        // can be computed as
        // L(x) = l(x) \sum_{j=0}^{k-1} w_j y_j / (x - x_j),
        // where
        // l(x) = \prod_{j=0}^{k-1} (x - x_j),
        // and
        // w_j = \prod_{m \neq j} (x_j - x_m)^{-1}.
        //
        // This function replaces in-place in self.buffer the values y_j by the
        // terms w_j y_j. This speeds up evaluation of the L(x) for encoding
        // each FEC packet.
        let k = self.num_systematic();
        for j in 0..k {
            // Compute w_j
            let wj = GF64K::from(1) / wj_inv(j, k);
            // Multiply each y_j by w_j
            let data = self.buffer[usize::from(j)].data_as_mut();
            for word in data.chunks_exact_mut(2) {
                let word: &mut [u8; 2] = word.try_into().unwrap();
                let yj = GF64K::from(u16::from_be_bytes(*word));
                let yj_wj = yj * wj;
                *word = u16::from(yj_wj).to_be_bytes();
            }
        }
    }

    /// Generate the packet with a corresponding `packet_id`.
    ///
    /// If the `packet_id` is smaller than the number of systematic packets in
    /// the image, the corresponding systematic packet give to [`Encoder::new`]
    /// is generated. Otherwise, a FEC packet is generated. The packet is
    /// written to `output`.
    pub fn encode(&self, packet_id: u16, output: &mut S) {
        self.encode_header(packet_id, output);
        if output.is_fec_packet() {
            self.encode_fec_data(packet_id, output.data_as_mut());
        } else {
            self.encode_systematic_data(packet_id, output.data_as_mut());
        }
        output.update_crc32();
    }

    fn encode_header(&self, packet_id: u16, output: &mut S) {
        output.set_fixed_fields();
        output.callsign_as_mut().copy_from_slice(self.callsign());
        output.set_image_id(self.image_id());
        output.set_packet_id(packet_id);
        let is_fec = packet_id >= self.num_systematic();
        if is_fec {
            output.set_number_systematic_packets(self.num_systematic());
        } else {
            output.set_width(self.image_width());
            output.set_height(self.image_height());
        }
        output.set_flags(self.flags());
        output.set_eoi(packet_id == self.num_systematic() - 1);
        output.set_fec_packet(is_fec);
    }

    fn encode_fec_data(&self, packet_id: u16, data: &mut GenericArray<u8, S::DataLen>) {
        // See values_to_lagrange for the formulas
        let x = GF64K::from(packet_id);
        let k = self.num_systematic();
        // Compute l(x)
        let mut lx = GF64K::from(1);
        for j in 0..k {
            let xj = GF64K::from(j);
            lx *= x - xj;
        }

        // Compute \sum_{j=0}^{k-1} w_j y_j / (x - x_j) for each word in the
        // output data
        for (r, word) in data.chunks_exact_mut(2).enumerate() {
            let mut sum = GF64K::from(0);
            for (j, wj_yj_s) in self.buffer.iter().map(|packet| packet.data()).enumerate() {
                let wj_yj = GF64K::from(u16::from_be_bytes(
                    wj_yj_s[2 * r..2 * r + 2].try_into().unwrap(),
                ));
                let xj = GF64K::from(j as u16);
                sum += wj_yj / (x - xj);
            }
            let word: &mut [u8; 2] = word.try_into().unwrap();
            let result = lx * sum;
            *word = u16::from(result).to_be_bytes();
        }
    }

    fn encode_systematic_data(&self, packet_id: u16, data: &mut GenericArray<u8, S::DataLen>) {
        // The algorithm in encode_fec_data is not valid for systematic packets,
        // because both l(x) and one of the terms 1 / (x - x_j) vanish. In the
        // systematic case we compute w_j again and divide, undoing what we did
        // in values_to_lagrange.
        let wjinv = wj_inv(packet_id, self.num_systematic());
        for (word_in, word_out) in self.buffer[usize::from(packet_id)]
            .data()
            .chunks_exact(2)
            .zip(data.chunks_exact_mut(2))
        {
            let wj_yj = GF64K::from(u16::from_be_bytes(word_in.try_into().unwrap()));
            let yj = wj_yj * wjinv;
            let word_out: &mut [u8; 2] = word_out.try_into().unwrap();
            *word_out = u16::from(yj).to_be_bytes();
        }
    }

    fn callsign(&self) -> &GenericArray<u8, S::CallsignLen> {
        self.buffer[0].callsign()
    }

    fn num_systematic(&self) -> u16 {
        self.buffer.len() as u16
    }

    fn image_id(&self) -> u8 {
        self.buffer[0].image_id()
    }

    fn image_width(&self) -> u8 {
        self.buffer[0].width().unwrap()
    }

    fn image_height(&self) -> u8 {
        self.buffer[0].height().unwrap()
    }

    fn flags(&self) -> u8 {
        self.buffer[0].flags()
    }
}

/// SSDV FEC decoder.
///
/// This struct represents the FEC decoder. The way to use the FEC decoder is
/// through the [`Decoder::decode`] associated function. The struct only exists
/// for namespacing this function.
#[derive(Debug)]
pub struct Decoder {}

#[derive(Debug)]
struct DecoderHelper<'a, 'b, S: SSDVPacket> {
    input: &'a mut [S],
    output: &'b mut [S],
    callsign: GenericArray<u8, S::CallsignLen>,
    num_systematic: u16,
    image_id: u8,
    image_width: u8,
    image_height: u8,
    flags: u8,
}

/// Error produced by the SSDV FEC decoder.
///
/// This enum lists the errors that can be produced by [`Decoder`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum DecoderError {
    /// The EOI flag is set on a FEC packet.
    #[cfg_attr(feature = "std", error("EOI set on FEC packet"))]
    EoiOnFecPacket,
    /// The EOI flag is set on several different systematic packets.
    #[cfg_attr(feature = "std", error("EOI set on several different packets"))]
    DuplicatedEoi,
    /// There are different FEC packets containing a different value in the
    /// number of systematic packets field.
    #[cfg_attr(
        feature = "std",
        error("mismatched number of systematic packets on different FEC packets")
    )]
    NumSystematicMismatch,
    /// The number of systematic packets in the image could not be determined.
    ///
    /// This happens if the last systematic packet (carrying the EOI flag) is
    /// missing and there are no FEC packets.
    #[cfg_attr(
        feature = "std",
        error("could not determine number of systematic packets")
    )]
    UnknownNumSystematic,
    /// There is a mismatch between the packet ID of the packet carrying the EOI
    /// flag and the number of systematic packets field in the FEC packets.
    #[cfg_attr(
        feature = "std",
        error("mismatch between EOI and number of systematic packets")
    )]
    EoiFecMismatch,
    /// There are not enough input packets for decoding.
    ///
    /// The decoder needs as least as many distinct input packets as systematic
    /// packets are there in the image.
    #[cfg_attr(feature = "std", error("not enough input packets"))]
    NotEnoughInput,
    /// The output buffer is too short.
    ///
    /// The length of the output buffer must be greater or equal than the number
    /// of systematic packets in the image.
    #[cfg_attr(feature = "std", error("output buffer is too short"))]
    OutputTooShort,
    /// A systematic packet has a wrong packet ID.
    #[cfg_attr(feature = "std", error("wrong packet ID on systematic packet"))]
    WrongSystematicId,
    /// There are multiple image IDs in the packets.
    #[cfg_attr(feature = "std", error("multiple image IDs"))]
    MultipleImageIds,
    /// There are different packets with inconsistent values of the flags field.
    #[cfg_attr(feature = "std", error("inconsistent flags on different packets"))]
    InconsistentFlags,
    /// There are systematic packets with different values of the image width or
    /// height.
    #[cfg_attr(
        feature = "std",
        error("mismatched width or height on different systematic packets")
    )]
    DimensionsMismatch,
    /// There are no systematic packets.
    ///
    /// At least one systematic packet is required to obtain the image width and
    /// height.
    #[cfg_attr(feature = "std", error("no systematic packets"))]
    NoSystematic,
}

impl Decoder {
    /// Decodes a list of SSDV packets to obtain the original SSDV image.
    ///
    /// This function receives a slice `input` containing SSDV packets from a
    /// single image, and, if possible, obtains the original SSDV image and
    /// writes the results to the beginning of the `output` slice, returning the
    /// subslice of `output` that contains the image packets. If decoding is not
    /// possible, the function returns an error.
    ///
    /// The packets in `input` can be in any order and can have duplicates. The
    /// function works in-place in the `input` slice, modifying its contents.
    pub fn decode<'a, S: SSDVPacket>(
        input: &mut [S],
        output: &'a mut [S],
    ) -> Result<&'a mut [S], DecoderError> {
        let mut decoder = DecoderHelper::new(input, output)?;
        decoder.init_output();
        decoder.copy_systematic();
        if !decoder.all_systematic_obtained() {
            decoder.values_to_lagrange();
            decoder.interpolate_missing();
        }
        Ok(&mut decoder.output[..usize::from(decoder.num_systematic)])
    }
}

impl<'a, 'b, S: SSDVPacket> DecoderHelper<'a, 'b, S> {
    fn new(input: &'a mut [S], output: &'b mut [S]) -> Result<Self, DecoderError> {
        let input = Self::remove_duplicates_and_wrong_crcs(input);
        let num_systematic = Self::find_num_systematic(input)?;
        // Check here that !input.is_empty(), since we will access to input[0]
        // below. In principle, num_systematic should be > 0, but if the packets
        // are maliciously formed, perhaps it might be computed as 0 by the
        // decoder.
        if input.is_empty() || input.len() < usize::from(num_systematic) {
            return Err(DecoderError::NotEnoughInput);
        }
        if output.len() < usize::from(num_systematic) {
            return Err(DecoderError::OutputTooShort);
        }
        let callsign = input[0].callsign().clone();
        Self::check_systematic_ids(input, num_systematic)?;
        let (image_id, flags) = Self::find_image_id_flags(input)?;
        let (image_width, image_height) = Self::find_image_dimensions(input)?;
        Ok(DecoderHelper {
            input,
            output,
            callsign,
            num_systematic,
            image_id,
            image_width,
            image_height,
            flags,
        })
    }

    fn remove_duplicates_and_wrong_crcs(input: &mut [S]) -> &mut [S] {
        let mut len = input.len();
        let mut j = 0;
        while j < len {
            if !input[j].crc32_is_valid() {
                // remove wrong CRC
                input.copy_within(j + 1..len, j);
                len -= 1;
                continue;
            }
            let id = input[j].packet_id();
            let mut k = j + 1;
            while k < len {
                if input[k].packet_id() == id {
                    // remove duplicate
                    input.copy_within(k + 1..len, k);
                    len -= 1;
                } else {
                    k += 1;
                }
            }
            j += 1;
        }
        &mut input[..len]
    }

    fn find_num_systematic(input: &[S]) -> Result<u16, DecoderError> {
        let mut id_eoi = None;
        let mut from_fec_packets = None;
        for packet in input {
            if packet.is_eoi() {
                if packet.is_fec_packet() {
                    return Err(DecoderError::EoiOnFecPacket);
                }
                if id_eoi.is_some() {
                    return Err(DecoderError::DuplicatedEoi);
                }
                id_eoi = Some(packet.packet_id());
            }
            if let Some(k) = packet.number_systematic_packets() {
                if let Some(k2) = from_fec_packets {
                    if k != k2 {
                        return Err(DecoderError::NumSystematicMismatch);
                    }
                } else {
                    from_fec_packets = Some(k);
                }
            }
        }
        match (id_eoi, from_fec_packets) {
            (None, None) => Err(DecoderError::UnknownNumSystematic),
            (Some(k), None) => Ok(k + 1),
            (None, Some(k)) => Ok(k),
            (Some(k), Some(k2)) => {
                if k + 1 == k2 {
                    Ok(k2)
                } else {
                    Err(DecoderError::EoiFecMismatch)
                }
            }
        }
    }

    fn check_systematic_ids(input: &[S], num_systematic: u16) -> Result<(), DecoderError> {
        for packet in input {
            if !packet.is_fec_packet() && packet.packet_id() >= num_systematic {
                return Err(DecoderError::WrongSystematicId);
            }
        }
        Ok(())
    }

    fn find_image_id_flags(input: &[S]) -> Result<(u8, u8), DecoderError> {
        let image_id = input[0].image_id();

        fn clean_flags(flags: u8) -> u8 {
            // remove EOI and FEC packet flags
            flags & !0x44
        }

        let flags = clean_flags(input[0].flags());

        for packet in input {
            if packet.image_id() != image_id {
                return Err(DecoderError::MultipleImageIds);
            }
            if clean_flags(packet.flags()) != flags {
                return Err(DecoderError::InconsistentFlags);
            }
        }
        Ok((image_id, flags))
    }

    fn find_image_dimensions(input: &[S]) -> Result<(u8, u8), DecoderError> {
        let mut dimensions = None;
        for packet in input {
            if let Some(width) = packet.width() {
                // if width is present, then height is also present
                let height = packet.height().unwrap();
                if let Some((w, h)) = dimensions {
                    if w != width || h != height {
                        return Err(DecoderError::DimensionsMismatch);
                    }
                } else {
                    dimensions = Some((width, height))
                }
            }
        }
        dimensions.ok_or(DecoderError::NoSystematic)
    }

    fn init_output(&mut self) {
        for packet in self.output.iter_mut() {
            // this lets us know that the packet has not been recovered yet
            packet.set_packet_id(Self::INVALID_PACKET_ID);
        }
    }

    const INVALID_PACKET_ID: u16 = 0xffff;

    fn copy_systematic(&mut self) {
        for packet in self.input.iter() {
            if !packet.is_fec_packet() {
                let id = packet.packet_id();
                self.output[usize::from(id)].clone_from(packet);
            }
        }
    }

    fn all_systematic_obtained(&self) -> bool {
        !self.output[..usize::from(self.num_systematic)]
            .iter()
            .any(|&packet| packet.packet_id() == Self::INVALID_PACKET_ID)
    }

    // Computes
    // w_j^{-1} = \prod_{m \neq j} (x_j - x_m).
    //
    // This is different from the wj_inv function used in Encoder because the
    // packet_id's of the first k packets in the input buffer are not
    // sequential.
    fn wj_inv(&self, j: usize) -> GF64K {
        let xj = GF64K::from(self.input[j].packet_id());
        let mut ret = GF64K::from(1);
        for (m, p) in self.input[0..usize::from(self.num_systematic)]
            .iter()
            .enumerate()
        {
            if m != j {
                let xm = GF64K::from(p.packet_id());
                ret *= xj - xm;
            }
        }
        ret
    }

    fn values_to_lagrange(&mut self) {
        // See Encoder::values_to_lagrange
        for j in 0..usize::from(self.num_systematic) {
            let wj = GF64K::from(1) / self.wj_inv(j);
            let data = self.input[j].data_as_mut();
            for word in data.chunks_exact_mut(2) {
                let word: &mut [u8; 2] = word.try_into().unwrap();
                let yj = GF64K::from(u16::from_be_bytes(*word));
                let yj_wj = yj * wj;
                *word = u16::from(yj_wj).to_be_bytes();
            }
        }
    }

    fn interpolate_missing(&mut self) {
        // See Encoder::encode_fec_data
        let k = usize::from(self.num_systematic);
        for (j, packet) in self.output[..k]
            .iter_mut()
            .enumerate()
            .filter(|(_, packet)| packet.packet_id() == Self::INVALID_PACKET_ID)
        {
            // Compute l(x)
            let x = GF64K::from(j as u16);
            let mut lx = GF64K::from(1);
            for p in &self.input[..k] {
                let xj = GF64K::from(p.packet_id());
                lx *= x - xj;
            }

            // Compute \sum_{j=0}^{k-1} w_j y_j / (x - x_j) for each word in the
            // output data
            let data = packet.data_as_mut();
            for (r, word) in data.chunks_exact_mut(2).enumerate() {
                let mut sum = GF64K::from(0);
                for p in &self.input[..k] {
                    let wj_yj_s = p.data();
                    let wj_yj = GF64K::from(u16::from_be_bytes(
                        wj_yj_s[2 * r..2 * r + 2].try_into().unwrap(),
                    ));
                    let xj = GF64K::from(p.packet_id());
                    sum += wj_yj / (x - xj);
                }
                let word: &mut [u8; 2] = word.try_into().unwrap();
                let result = lx * sum;
                *word = u16::from(result).to_be_bytes();
            }

            // Fill header
            packet.set_fixed_fields();
            packet.callsign_as_mut().copy_from_slice(&self.callsign);
            packet.set_image_id(self.image_id);
            packet.set_packet_id(j as u16);
            packet.set_width(self.image_width);
            packet.set_height(self.image_height);
            packet.set_flags(self.flags);
            packet.set_eoi(j == k - 1);
            packet.set_fec_packet(false);

            // Fill CRC32
            packet.update_crc32();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        SSDVParameters,
        packet_formats::longjiang2::{Parameters, SSDVPacket},
        test_data::IMG_230_SSDV,
    };
    use generic_array::typenum::Unsigned;

    const PACKET_LEN: usize = <Parameters as SSDVParameters>::PacketLen::USIZE;

    #[test]
    fn encode_img_230_systematic() {
        let mut ssdv = IMG_230_SSDV
            .chunks_exact(PACKET_LEN)
            .map(|chunk| SSDVPacket::new_from_slice(chunk).unwrap())
            .collect::<Vec<SSDVPacket>>();
        let encoder = Encoder::new(&mut ssdv).unwrap();

        let mut encoded_packet = SSDVPacket::default();
        for (j, packet) in IMG_230_SSDV.chunks_exact(PACKET_LEN).enumerate() {
            let original_packet = SSDVPacket::new_from_slice(packet).unwrap();
            encoder.encode(u16::try_from(j).unwrap(), &mut encoded_packet);
            assert_eq!(&encoded_packet, &original_packet);
        }
    }

    #[test]
    fn encode_decode_img_230_one_every_n() {
        let ssdv = IMG_230_SSDV
            .chunks_exact(PACKET_LEN)
            .map(|chunk| SSDVPacket::new_from_slice(chunk).unwrap())
            .collect::<Vec<SSDVPacket>>();
        let k = ssdv.len();
        // Do a copy to keep ssdv as a reference (since the encoder destroys the input)
        let mut ssdv_copy = ssdv.clone();
        let encoder = Encoder::new(&mut ssdv_copy).unwrap();

        for one_in_every in 1..=10 {
            let mut encoded_packets = (0..one_in_every * k)
                .step_by(one_in_every)
                .map(|j| {
                    let mut encoded_packet = SSDVPacket::default();
                    encoder.encode(u16::try_from(j).unwrap(), &mut encoded_packet);
                    encoded_packet
                })
                .collect::<Vec<SSDVPacket>>();

            let mut output = vec![SSDVPacket::default(); k];
            Decoder::decode(&mut encoded_packets[..], &mut output[..]).unwrap();
            for (j, (s, o)) in ssdv.iter().zip(output.iter()).enumerate() {
                assert_eq!(
                    &s, &o,
                    "SSDV packet {j} is different \
                     (decoding with one in every {one_in_every} packet received)"
                );
            }
        }
    }
}
