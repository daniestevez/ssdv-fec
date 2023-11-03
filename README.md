# SSDV systematic erasure FEC

This repository contains a systematic erasure FEC scheme for
[SSDV](https://github.com/fsphil/ssdv). The FEC scheme is based on a
Reed-Solomon code over GF(2¹⁶) used as a fountain-like code. This idea was first
described in the blog post [An erasure FEC for
SSDV](https://destevez.net/2023/05/an-erasure-fec-for-ssdv/) by the author of
this crate.

The repository is organized as follows:

- [ssdv-fec](ssdv-fec). The implementation of the FEC scheme as a Rust crate,
  which can be used as a library or as a CLI application. See this first.

- [erminaz-ssdv-fec](erminaz-ssdv-fec). A wrapper of the FEC library for the
  flight software of the AMSAT-DL ERMINAZ mission. It is built as a static
  library with a C API for ARM Cortex-M4.
