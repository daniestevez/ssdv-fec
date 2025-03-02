# SSDV systematic erasure FEC

This Rust crate implements a systematic erasure FEC scheme for
[SSDV](https://github.com/fsphil/ssdv). The FEC scheme is based on a
Reed-Solomon code over GF(2¹⁶) used as a fountain-like code. This idea was first
described in the blog post
[An erasure FEC for SSDV](https://destevez.net/2023/05/an-erasure-fec-for-ssdv/)
by the author of this crate.

Given an SSDV image formed by k SSDV packets, the FEC encoder can generate up to
2¹⁶ different SSDV packets identified by a packet ID from 0 to 2¹⁶-1. The
packets with IDs from 0 to k-1 are called "systematic packets" and are the
same as the k packets of the original image. The remaining packets are called
"FEC packets". Each packet can be generated on demand according to the needs of
the transmitter. The large amount of 2¹⁶ distinct packets than can be generated
provides a virtually limitless source of packets. The receiver can recover the
original SSDV image from any set of k distinct packets.

Several SSDV packet formats are supported:

- The no-FEC
  [standard packet format](https://ukhas.org.uk/doku.php?id=guides:ssdv#packet_format)
  implemented by the
  [upstream SSDV](https://github.com/fsphil/ssdv). This is a 256-byte packet format
  that includes a callsign.

- The custom packet format used by Longjiang-2, which is implemented in a
  [fork of SSVD](https://github.com/daniestevez/ssdv). This is a 218-byte packet
  format that omits the sync byte, packet type and callsign fields (but includes
  them implicitly in the generation of the CRC-32).

Other packet formats can be supported by implementing the `SSDVParameters` or
the `SSDVPacket` trait.

The crate supports `no_std` and the implementation is designed with small
microcontrollers in mind. The GF(2¹⁶) arithmetic only uses two tables of 256
bytes each that are included in the `.rodata` section. The FEC encoder and
decoder work with externally provided slices, giving freedom as to how to
perform memory allocation, and do the computations in-place when possible. The
memory required for encoding corresponds to a buffer containing the k SSDV
packets of the original image, and a buffer containing the packet being
encoded. The memory required for decoding corresponds to a buffer containing at
least k distinct received SSDV packets, and another buffer where the k SSDV
packets that compose the original image can be written. Besides these buffers,
the algorithms use only a small amount of stack space.

A simple CLI application that can perform encoding and decoding can be built
with the `cli` feature, which is enabled by default.

## CLI application usage

The CLI application can be installed using

```
cargo install ssdv-fec
```

The `ssdv-fec` application supports the commands `encode` and `decode`.  The
SSDV packet format can be specified for both commands using the `--format`
argument. The default format is the standard no-FEC SSDV packet format.

To perform encoding, it is necessary to specify the number of packets to
generate in the output. This can be done with the `--npackets` argument to
specify a fixed number of packets, or with the `--rate` argument to specify the
coding rate. If `--rate` is used, the number of encoded packets is equal to the
number of packets in the original image divided by the coding rate (which must
be between 0 and 1). An example SSDV image that uses the Longjiang-2 packet
format can be found in the [`src/test_data`](src/test_data) directory. These are
examples of encoding.

```
ssdv-fec --format longjiang2 encode --rate 0.8 src/test_data/img_230.ssdv encoded.ssdv
ssdv-fec --format longjiang2 encode --npackets 256 src/test_data/img_230.ssdv encoded.ssdv
ssdv-fec --format longjiang2 encode --first 57 --npackets 15 src/test_data/img_230.ssdv encoded.ssdv
```

By default the packet ID of the first encoded packet is zero, but another first
packet ID can be chosen with the `--first` argument. The remaining packets use
consecutive packet IDs. The `--first` argument can be used to encode an
additional set of packets distinct from the previously encoded packets.

Decoding only requires the input file and output file as arguments. Here is an
example of decoding.

```
ssdv-fec --format longjiang2 decode encoded.ssdv decoded.ssdv
```

The input file for decoding should only contain packets of a single image. The
packets can be in any order an they can be repeated. If decoding fails, the
application indicates the cause of the error.

## API documentation

The documentation for the ssdv-fec Rust crate is hosted in
[docs.rs](https://docs.rs/ssdv-fec/).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
