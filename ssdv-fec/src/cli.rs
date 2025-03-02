//! CLI application.
//!
//! This module implements the CLI application for encoding and decoding with
//! SSDV FEC.

use crate::{packet_formats, Decoder, Encoder, SSDVPacket, SSDVPacketArray, SSDVParameters};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::{
    convert::AsRef,
    fs::File,
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

/// SSDV FEC encoder and decoder.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// SSDV packet format.
    #[arg(long, default_value = "no-fec")]
    format: SSDVFormat,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, ValueEnum)]
enum SSDVFormat {
    /// No-FEC mode (256 byte packet) defined in standard SSDV.
    NoFec,
    /// Custom 218 byte packet format used by Longjiang-2
    Longjiang2,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Encode an SSDV FEC packet.
    Encode {
        /// First packet ID.
        #[arg(long, default_value_t = 0)]
        first: u16,
        /// Number of packets to encode.
        #[arg(long)]
        npackets: Option<u16>,
        /// Coding rate to use.
        ///
        /// Mutually exclusive with npackets.
        ///
        /// Chooses the number of packets as the number of packes in the input
        /// divided by the rate.
        #[arg(long)]
        rate: Option<f64>,
        /// Input file (original SSDV image).
        input: PathBuf,
        /// Output file (encoded SSDV packet).
        output: PathBuf,
    },
    /// Decode an SSDV FEC image.
    Decode {
        /// Input file (received SSDV FEC packets).
        input: PathBuf,
        /// Output file (recovered SSDV image).
        output: PathBuf,
    },
}

/// Runs the CLI application.
pub fn run() -> Result<()> {
    let args = Args::parse();
    match args.format {
        SSDVFormat::NoFec => {
            run_with_packet_parameters::<packet_formats::no_fec::Parameters>(&args)
        }
        SSDVFormat::Longjiang2 => {
            run_with_packet_parameters::<packet_formats::longjiang2::Parameters>(&args)
        }
    }
}

fn run_with_packet_parameters<P: SSDVParameters>(args: &Args) -> Result<()> {
    match &args.command {
        &Command::Encode {
            first,
            npackets,
            rate,
            ref input,
            ref output,
        } => {
            match (npackets, rate) {
                (Some(_), Some(_)) => {
                    anyhow::bail!("the --nargs and --rate options are mutually exclusive")
                }
                (None, None) => anyhow::bail!("one of the --nargs and --rate options must be used"),
                (_, Some(rate)) if rate <= 0.0 || rate > 1.0 => {
                    anyhow::bail!("the coding rate must be in the interval (0, 1]")
                }
                _ => (),
            };
            let mut input = read_ssdv_to_vec(input)?;
            let input_len = input.len();
            let encoder = Encoder::new(&mut input)?;
            let npackets = match (npackets, rate) {
                (Some(npackets), None) => npackets,
                (None, Some(rate)) => u16::try_from(
                    ((input_len as f64 / rate).round() as u32).min(u32::from(u16::MAX - first)),
                )
                .unwrap(),
                _ => unreachable!(),
            };
            let mut encoded = vec![SSDVPacketArray::<P>::default(); usize::from(npackets)];
            for (j, packet) in encoded.iter_mut().enumerate() {
                let packet_id = first + j as u16;
                encoder.encode(packet_id, packet);
            }
            write_ssdv_slice(output, &encoded)?;
        }
        Command::Decode { input, output } => {
            let mut input = read_ssdv_to_vec(input)?;
            let mut output_vec = vec![SSDVPacketArray::<P>::default(); input.len()];
            let decoded = Decoder::decode(&mut input, &mut output_vec)?;
            write_ssdv_slice(output, decoded)?;
        }
    }
    Ok(())
}

fn read_ssdv_to_vec<P: SSDVParameters>(path: impl AsRef<Path>) -> Result<Vec<SSDVPacketArray<P>>> {
    let mut file = File::open(path)?;
    let mut packets = Vec::new();
    for n in 0.. {
        let mut packet = SSDVPacketArray::default();
        match file.read_exact(&mut packet.0) {
            Err(err) if matches!(err.kind(), ErrorKind::UnexpectedEof) => return Ok(packets),
            Err(err) => Err(err)?,
            Ok(()) => (),
        }
        if !packet.crc32_is_valid() {
            eprintln!(
                "CRC-32 for packet number {n} in input file is wrong \
                 (perhaps the packet format is wrong)"
            );
        }
        packets.push(packet);
    }
    unreachable!();
}

fn write_ssdv_slice<P: SSDVParameters>(
    path: impl AsRef<Path>,
    ssdv_packets: &[SSDVPacketArray<P>],
) -> Result<()> {
    let mut file = File::create(path)?;
    for packet in ssdv_packets {
        file.write_all(&packet.0)?;
    }
    Ok(())
}
