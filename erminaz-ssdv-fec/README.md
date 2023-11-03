# SSDV FEC library for the AMSAT-DL ERMINAZ mission.

This crate contains a C API wrapper of the [`ssdv_fec`] crate as required by the
AMSAT-DL ERMINAZ mission flight software. The crate is prepared to build a
static library for an ARM Cortex-M4 using the `thumbv7em-none-eabi` target, and
a C header is generated using `cbindgen`.

## Building

The static library can be built with
```
cargo build --release
```

The library can then be found in `target/thumbv7em-none-eabi/release/liberminaz_ssdv_fec.a`,
and the header in `erminaz_ssdv_fec.h`.

The rust toolchain required to build this library can be installed with
[rustup](https://rustup.rs/). The `thumbv7em-none-eabi` needs to be installed by doing
```
rustup target add thumbv7em-none-eabi
```

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
