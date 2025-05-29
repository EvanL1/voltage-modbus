# Voltage Modbus

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![GitHub](https://img.shields.io/badge/github-voltage_modbus-blue.svg)](https://github.com/voltage-llc/voltage_modbus)

> **High-Performance Modbus TCP/RTU Library for Rust**
>
> **Author:** Evan Liu <evan.liu@voltageenergy.com>
> **Version:** 0.1.0
> **License:** MIT

A comprehensive, high-performance Modbus TCP/RTU implementation in pure Rust designed for industrial automation, IoT applications, and smart grid systems.

## ✨ Features

- **🚀 High Performance**: Async/await support with Tokio for maximum throughput
- **🔧 Complete Protocol Support**: Both Modbus TCP and RTU protocols
- **🛡️ Memory Safe**: Pure Rust implementation with zero unsafe code
- **⚡ Zero-Copy Operations**: Optimized for minimal memory allocations
- **🔄 Concurrent Processing**: Multi-client server support
- **📊 Built-in Monitoring**: Comprehensive statistics and metrics
- **🏭 Production Ready**: Extensive testing and error handling

## 📋 Supported Function Codes

| Code | Function                 | Client | Server |
| ---- | ------------------------ | ------ | ------ |
| 0x01 | Read Coils               | ✅     | ✅     |
| 0x02 | Read Discrete Inputs     | ✅     | ✅     |
| 0x03 | Read Holding Registers   | ✅     | ✅     |
| 0x04 | Read Input Registers     | ✅     | ✅     |
| 0x05 | Write Single Coil        | ✅     | ✅     |
| 0x06 | Write Single Register    | ✅     | ✅     |
| 0x0F | Write Multiple Coils     | ✅     | ✅     |
| 0x10 | Write Multiple Registers | ✅     | ✅     |

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
voltage_modbus = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Client Example

```rust
use voltage_modbus::{ModbusClient, ModbusResult};
use std::time::Duration;

#[tokio::main]
async fn main() -> ModbusResult<()> {
    // Connect to Modbus TCP server
    let address = "127.0.0.1:502".parse().unwrap();
    let timeout = Duration::from_secs(5);
    let mut client = ModbusClient::new_tcp(address, timeout).await?;
  
    // Read holding registers
    let values = client.read_holding_registers(1, 0, 10).await?;
    println!("Read registers: {:?}", values);
  
    // Write single register
    client.write_single_register(1, 100, 0x1234).await?;
  
    client.close().await?;
    Ok(())
}
```

### Server Example

```rust
use voltage_modbus::{
    ModbusTcpServer, ModbusTcpServerConfig, ModbusServer, ModbusRegisterBank
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server configuration
    let config = ModbusTcpServerConfig {
        bind_address: "127.0.0.1:502".parse().unwrap(),
        max_connections: 50,
        request_timeout: Duration::from_secs(30),
        register_bank: Some(Arc::new(ModbusRegisterBank::new())),
    };
  
    // Start server
    let mut server = ModbusTcpServer::with_config(config)?;
    server.start().await?;
  
    // Server is now running...
    Ok(())
}
```

## 📖 Documentation

- **[GitHub Pages](https://voltage-llc.github.io/voltage_modbus/)** - Live documentation (auto-updated)
- **[API Reference](https://docs.rs/voltage_modbus)** - Complete API documentation
- **[Examples](./examples/)** - Usage examples and tutorials
- **[Performance Guide](./docs/performance.md)** - Optimization tips
- **[Protocol Reference](./docs/protocol.md)** - Modbus protocol details

## 🏗️ Architecture

```text
┌─────────────────┐    ┌─────────────────┐
│   Application   │    │   Application   │
└─────────────────┘    └─────────────────┘
         │                       │
┌─────────────────┐    ┌─────────────────┐
│  Modbus Client  │    │  Modbus Server  │
└─────────────────┘    └─────────────────┘
         │                       │
┌─────────────────┐    ┌─────────────────┐
│   Protocol      │    │ Register Bank   │
│   (TCP/RTU)     │    │   (Storage)     │
└─────────────────┘    └─────────────────┘
         │                       │
┌─────────────────┐    ┌─────────────────┐
│   Transport     │◄──►│   Transport     │
│   (Async I/O)   │    │   (Async I/O)   │
└─────────────────┘    └─────────────────┘
```

### Core Modules

- **`error`** - Error types and result handling
- **`protocol`** - Modbus protocol definitions and message handling
- **`transport`** - Network transport layer for TCP and RTU communication
- **`client`** - Modbus client implementations
- **`server`** - Modbus server implementations with concurrent support
- **`register_bank`** - Thread-safe register storage for server applications
- **`utils`** - Utility functions and performance monitoring

## 🧪 Examples and Testing

### Run Examples

```bash
# Start the demo server
cargo run --bin server_demo

# Run client demo
cargo run --bin demo

# Run full function tests
cargo run --bin full_function_test

# Run performance tests
cargo run --bin advanced_test

# Run all tests
./test_server.sh
```

### Test Coverage

- ✅ All Modbus function codes
- ✅ Error handling and recovery
- ✅ Concurrent client connections
- ✅ Protocol compliance testing
- ✅ Performance benchmarking

## 📈 Performance

### Benchmarks

| Metric                           | Value              |
| -------------------------------- | ------------------ |
| **Latency**                | < 1ms (local)      |
| **Throughput**             | 1000+ requests/sec |
| **Concurrent Connections** | 50+ clients        |
| **Memory Usage**           | < 10MB (baseline)  |
| **CPU Usage**              | < 5% (idle)        |

### Optimization Features

- **Async I/O**: Non-blocking operations with Tokio
- **Connection Pooling**: Efficient connection management
- **Zero-Copy**: Minimal memory allocations
- **Lock-Free Operations**: Where possible
- **Configurable Timeouts**: Adaptive timeout management

## 🔧 Configuration

### Client Configuration

```rust
use voltage_modbus::{ModbusClient};
use std::time::Duration;

let address = "192.168.1.100:502".parse().unwrap();
let timeout = Duration::from_secs(10);
let mut client = ModbusClient::new_tcp(address, timeout).await?;
```

### Server Configuration

```rust
use voltage_modbus::{ModbusTcpServerConfig, ModbusRegisterBank};
use std::sync::Arc;

let config = ModbusTcpServerConfig {
    bind_address: "0.0.0.0:502".parse().unwrap(),
    max_connections: 100,
    request_timeout: Duration::from_secs(30),
    register_bank: Some(Arc::new(ModbusRegisterBank::with_sizes(
        10000, // coils
        10000, // discrete_inputs  
        10000, // holding_registers
        10000, // input_registers
    ))),
};
```

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/voltage-llc/voltage_modbus.git
cd voltage_modbus
cargo build --release
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Documentation tests
cargo test --doc

# All tests with output
cargo test -- --nocapture
```

### Generating Documentation

```bash
# Generate and open documentation (no dependencies)
cargo doc --no-deps --open

# Generate with all features (no dependencies)  
cargo doc --no-deps --all-features --open
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. **Clone the repository**
2. **Install Rust** (latest stable)
3. **Install dependencies**: `cargo build`
4. **Run tests**: `cargo test`
5. **Check formatting**: `cargo fmt --check`
6. **Run linter**: `cargo clippy`

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Modbus Organization** for the protocol specification
- **Tokio Team** for the excellent async runtime
- **Rust Community** for the amazing ecosystem

## 📞 Support

- **Documentation**: https://docs.rs/voltage_modbus
- **Issues**: https://github.com/voltage-llc/voltage_modbus/issues
- **Discussions**: https://github.com/voltage-llc/voltage_modbus/discussions
- **Email**: evan.liu@voltageenergy.com

---

**Built with ❤️ by Evan Liu for the Rust and Industrial Automation communities.**
