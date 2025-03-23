# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-03-23

### Added

- Support for multiple SSDV packet formats.

### Changed

- Check CRC of input packets in CLI tool.

### Removed

- Removed ssdv-fec-gf-tables proc macro crate. Table generation moved to build
  script.

## [0.1.1] - 2024-10-12

### Changed

- Fixed typo in README and rustdoc.

## [0.1.0] - 2023-11-03

- Initial release.

[unreleased]: https://github.com/daniestevez/ssdv-fec/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/daniestevez/ssdv-fec/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/daniestevez/ssdv-fec/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/daniestevez/ssdv-fec/releases/tag/v0.1.0
