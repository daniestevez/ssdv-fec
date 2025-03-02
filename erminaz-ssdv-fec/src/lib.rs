//! # SSDV FEC library for the AMSAT-DL ERMINAZ mission.
//!
//! This crate contains a C API wrapper of the [`ssdv_fec`] crate as required by
//! the AMSAT-DL ERMINAZ mission flight software. The crate is prepared to build
//! a static library for an ARM Cortex-M4 using the `thumbv7em-none-eabi`
//! target, and a C header is generated using `cbindgen`.

#![no_std]

extern crate panic_halt;

use core::{
    ffi::{c_char, c_int},
    mem::MaybeUninit,
    slice,
};
use ssdv_fec::{
    packet_formats::longjiang2::SSDVPacket, Decoder, DecoderError, Encoder, EncoderError,
};

static mut SSDV_FEC_ENCODER: MaybeUninit<Encoder<SSDVPacket>> = MaybeUninit::uninit();

/// Prepares the SSDV FEC encoder.
///
/// The `ssdv_packets` parameter should point to an array that contains the
/// concatenation of the SSDV systematic packets corresponding to a single
/// image. The number of packets in this array is indicated in the
/// `num_ssdv_packets` parameter.
///
/// The function returns zero on success, or a negative error code if there is
/// an error.
///
/// This function modifies the contents of the `ssdv_packets` array.
///
/// # Safety
///
/// This function is not thread safe. It cannot be called concurrently with
/// itself or with [`ssdv_fec_encoder_encode`]. The buffer pointed to by
/// `ssdv_packets` must have allocated storage for at least `num_ssdv_packets`
/// SSDV packets, must outlive all the usage of the FEC encoder until this
/// function is called again with a new buffer, and its contents must not be
/// accessed by other code while the buffer is owned by the FEC encoder.
#[no_mangle]
pub unsafe extern "C" fn ssdv_fec_encoder_setup(
    ssdv_packets: *mut c_char,
    num_ssdv_packets: c_int,
) -> c_int {
    let ssdv_packets =
        slice::from_raw_parts_mut(ssdv_packets.cast::<SSDVPacket>(), num_ssdv_packets as usize);
    let encoder = match Encoder::new(ssdv_packets) {
        Ok(encoder) => encoder,
        Err(err) => {
            return match err {
                EncoderError::EmptyInput => SSDV_FEC_ENCODER_ERR_EMPTY_INPUT,
                EncoderError::TooLongInput => SSDV_FEC_ENCODER_ERR_TOO_LONG_INPUT,
                EncoderError::NonSystematicInput => SSDV_FEC_ENCODER_ERR_NON_SYSTEMATIC_INPUT,
            }
        }
    };
    // SAFETY:
    //
    // According to the precodintions of this function, it is not to be called
    // concurrently with other functions that may access SSDV_FEC_ENCODER, so it
    // is safe to create a mutable reference to the static mut SSDV_FEC_ENCODER
    // here.
    //
    // https://doc.rust-lang.org/nightly/edition-guide/rust-2024/static-mut-references.html#safe-references
    let ssdv_fec_encoder_ptr = &raw mut SSDV_FEC_ENCODER;
    (*ssdv_fec_encoder_ptr).write(encoder);
    0
}

/// Generates a FEC encoded packet.
///
/// This function generates a systematic or FEC SSDV packet using the encoder
/// previously prepared by a call to [`ssdv_fec_encoder_setup`]. The `packet_id`
/// parameter corresponds to the SSDV packet ID. The `output` parameter should
/// point to an array of size at least the size of an SSDV packet. The encoded
/// packet is written to this array.
///
/// # Safety
///
/// This function is not thread safe. It cannot be called concurrently with
/// itself or with [`ssdv_fec_encoder_setup`]. The `packet_id` parameter must be
/// non-negative and smaller than `2**16 - 1`. The `output` buffer must have
/// allocated storage for at least one SSDV packet. All the safety
/// considerations of [`ssdv_fec_encoder_setup`] also apply.
#[no_mangle]
pub unsafe extern "C" fn ssdv_fec_encoder_encode(packet_id: c_int, output: *mut c_char) {
    let output = output.cast::<SSDVPacket>();
    let output = &mut *output;
    // SAFETY:
    //
    // According to the precodintions of this function, it is not to be called
    // concurrently with other functions that may access SSDV_FEC_ENCODER, so it
    // is safe to create a mutable reference to the static mut SSDV_FEC_ENCODER
    // here.
    //
    // https://doc.rust-lang.org/nightly/edition-guide/rust-2024/static-mut-references.html#safe-references
    let ssdv_fec_encoder_ptr = &raw mut SSDV_FEC_ENCODER;
    (*ssdv_fec_encoder_ptr)
        .assume_init_mut()
        .encode(packet_id as u16, output);
}

/// Decodes a FEC encoded SSDV image.
///
/// This function decodes an SSDV image from a series of FEC encoded SSDV
/// packets if there are enough packets to recover the original image. The
/// `input` parameter should point to an array that contains the FEC encoded
/// SSDV packets. The number of packets in this array is indicated by the
/// `num_input_packets` parameter. The `output` parameter should point to an
/// array where the decoded SSDV packets can be written to. The
/// `num_output_packets` indicates the length of this array, measured in number
/// of SSDV packets.
///
/// The function returns the length of the decoded SSDV image, measured in
/// number of SSDV packets, if decoding is successful. After the function
/// successfully returns, the beginning of the `output` array contains the
/// decoded SSDV image. If decoding is not possible, the function returns a
/// negative error code.
///
/// The function modifies the contents of the `input` array.
///
/// # Safety
///
/// The `input` and `output` buffers should be valid allocated storage of size
/// at least as indicated by their corresponding `num_*_packets` parameters.
#[no_mangle]
pub unsafe extern "C" fn ssdv_fec_decoder_decode(
    input: *mut c_char,
    num_input_packets: c_int,
    output: *mut c_char,
    num_output_packets: c_int,
) -> c_int {
    let input = slice::from_raw_parts_mut(input.cast::<SSDVPacket>(), num_input_packets as usize);
    let output =
        slice::from_raw_parts_mut(output.cast::<SSDVPacket>(), num_output_packets as usize);
    match Decoder::decode(input, output) {
        Ok(packets) => packets.len() as c_int,
        Err(err) => match err {
            DecoderError::EoiOnFecPacket => SSDV_FEC_DECODER_ERR_EOI_ON_FEC_PACKET,
            DecoderError::DuplicatedEoi => SSDV_FEC_DECODER_ERR_DUPLICATED_EOI,
            DecoderError::NumSystematicMismatch => SSDV_FEC_DECODER_ERR_NUM_SYSTEMATIC_MISMATCH,
            DecoderError::UnknownNumSystematic => SSDV_FEC_DECODER_ERR_UNKNOWN_NUM_SYSTEMATIC,
            DecoderError::EoiFecMismatch => SSDV_FEC_DECODER_ERR_EOI_FEC_MISMATCH,
            DecoderError::NotEnoughInput => SSDV_FEC_DECODER_ERR_NOT_ENOUGH_INPUT,
            DecoderError::OutputTooShort => SSDV_FEC_DECODER_ERR_OUTPUT_TOO_SHORT,
            DecoderError::WrongSystematicId => SSDV_FEC_DECODER_ERR_WRONG_SYSTEMATIC_ID,
            DecoderError::MultipleImageIds => SSDV_FEC_DECODER_ERR_MULTIPLE_IMAGE_IDS,
            DecoderError::InconsistentFlags => SSDV_FEC_DECODER_ERR_INCONSISTENT_FLAGS,
            DecoderError::DimensionsMismatch => SSDV_FEC_DECODER_ERR_DIMENSIONS_MISMATCH,
            DecoderError::NoSystematic => SSDV_FEC_DECODER_ERR_NO_SYSTEMATIC,
        },
    }
}

// Encoder error codes

/// Encoder input is empty
pub const SSDV_FEC_ENCODER_ERR_EMPTY_INPUT: c_int = -1;
/// Encoder input is too long
pub const SSDV_FEC_ENCODER_ERR_TOO_LONG_INPUT: c_int = -2;
/// Non-systematic packet in encoder input
pub const SSDV_FEC_ENCODER_ERR_NON_SYSTEMATIC_INPUT: c_int = -3;

// Decoder error codes

/// EOI set on FEC packet
pub const SSDV_FEC_DECODER_ERR_EOI_ON_FEC_PACKET: c_int = -16;
/// EOI set on several different packets
pub const SSDV_FEC_DECODER_ERR_DUPLICATED_EOI: c_int = -17;
/// Mismatched number of systematic packets on different FEC packets
pub const SSDV_FEC_DECODER_ERR_NUM_SYSTEMATIC_MISMATCH: c_int = -18;
/// Could not determine number of systematic packets
pub const SSDV_FEC_DECODER_ERR_UNKNOWN_NUM_SYSTEMATIC: c_int = -19;
/// Mismatch between EOI and number of systematic packets
pub const SSDV_FEC_DECODER_ERR_EOI_FEC_MISMATCH: c_int = -20;
/// Not enough input packets
pub const SSDV_FEC_DECODER_ERR_NOT_ENOUGH_INPUT: c_int = -21;
/// Output buffer is too short
pub const SSDV_FEC_DECODER_ERR_OUTPUT_TOO_SHORT: c_int = -22;
/// Wrong packet ID on systematic packet
pub const SSDV_FEC_DECODER_ERR_WRONG_SYSTEMATIC_ID: c_int = -23;
/// Multiple image IDs
pub const SSDV_FEC_DECODER_ERR_MULTIPLE_IMAGE_IDS: c_int = -24;
/// Inconsistent flags on different packets
pub const SSDV_FEC_DECODER_ERR_INCONSISTENT_FLAGS: c_int = -25;
/// Mismatched width or height on different systematic packets
pub const SSDV_FEC_DECODER_ERR_DIMENSIONS_MISMATCH: c_int = -26;
/// No systematic packets
pub const SSDV_FEC_DECODER_ERR_NO_SYSTEMATIC: c_int = -27;
