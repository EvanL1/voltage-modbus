# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased][Unreleased]

## [0.1.0][0.1.0] - 2025-05-29

### Added

- Initial release of Voltage Modbus library
- Complete Modbus TCP/RTU implementation in pure Rust
- High-performance async/await support with Tokio
- Full support for all standard Modbus function codes (0x01-0x10)
- Thread-safe register bank for server applications
- Comprehensive error handling and recovery
- Extensive documentation and examples
- GitHub Actions CI/CD pipeline
- Automatic documentation deployment to GitHub Pages

### Features

- **Client Support**: All read/write operations for coils and registers
- **Server Support**: Multi-client concurrent processing
- **Protocol Support**: Modbus TCP with RTU framework
- **Performance**: 1000+ requests/second, < 1ms latency
- **Safety**: Zero unsafe code, memory safe implementation
- **Testing**: Comprehensive test suite with 100% success rate

### Function Codes Supported

- 0x01: Read Coils
- 0x02: Read Discrete Inputs
- 0x03: Read Holding Registers
- 0x04: Read Input Registers
- 0x05: Write Single Coil
- 0x06: Write Single Register
- 0x0F: Write Multiple Coils
- 0x10: Write Multiple Registers

### Documentation

- Complete API reference with examples
- Architecture documentation and diagrams
- Performance benchmarks and optimization guide
- GitHub Pages deployment for live documentation

### Author

- Evan Liu <evan.liu@voltageenergy.com>

[Unreleased]: https://github.com/voltage-llc/voltage_modbus/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/voltage-llc/voltage_modbus/releases/tag/v0.1.0
