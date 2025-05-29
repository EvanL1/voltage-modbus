# Voltage Modbus API Documentation

**Author:** Evan Liu <evan.liu@voltageenergy.com>  
**Version:** 0.1.0  
**Generated:** $(date)

## 📚 Complete API Reference

The complete API documentation has been generated and is available at:
- **Local Documentation**: `target/doc/voltage_modbus/index.html`
- **Online Documentation**: https://docs.rs/voltage_modbus

## 🏗️ Module Structure

### Core Modules

#### `voltage_modbus::error`
- **`ModbusError`** - Comprehensive error type for all Modbus operations
- **`ModbusResult<T>`** - Type alias for `Result<T, ModbusError>`
- Error categories: Connection, Protocol, Timeout, InvalidData, etc.

#### `voltage_modbus::protocol`
- **`ModbusRequest`** - Request message structure
- **`ModbusResponse`** - Response message structure  
- **`ModbusFunction`** - Enumeration of all supported function codes
- Protocol constants and utilities

#### `voltage_modbus::transport`
- **`ModbusTransport`** - Trait for transport implementations
- **`TcpTransport`** - TCP transport implementation
- **`RtuTransport`** - RTU transport implementation (placeholder)
- **`TransportStats`** - Connection statistics

#### `voltage_modbus::client`
- **`ModbusClient`** - High-level client interface
- Async methods for all function codes
- Connection management and error handling

#### `voltage_modbus::server`
- **`ModbusServer`** - Server trait
- **`ModbusTcpServer`** - TCP server implementation
- **`ModbusTcpServerConfig`** - Server configuration
- **`ServerStats`** - Server statistics

#### `voltage_modbus::register_bank`
- **`ModbusRegisterBank`** - Thread-safe register storage
- **`RegisterBankStats`** - Storage statistics
- Support for all register types (coils, discrete inputs, holding/input registers)

#### `voltage_modbus::utils`
- **`PerformanceMetrics`** - Performance monitoring
- **`OperationTimer`** - Timing utilities
- Logging and debugging helpers

## 🚀 Quick API Reference

### Client Operations

```rust
use voltage_modbus::{ModbusClient, ModbusResult};
use std::time::Duration;

// Create client
let mut client = ModbusClient::new_tcp(address, timeout).await?;

// Read operations
let coils = client.read_coils(slave_id, address, count).await?;
let discrete_inputs = client.read_discrete_inputs(slave_id, address, count).await?;
let holding_registers = client.read_holding_registers(slave_id, address, count).await?;
let input_registers = client.read_input_registers(slave_id, address, count).await?;

// Write operations
client.write_single_coil(slave_id, address, value).await?;
client.write_single_register(slave_id, address, value).await?;
client.write_multiple_coils(slave_id, address, &values).await?;
client.write_multiple_registers(slave_id, address, &values).await?;

// Connection management
let stats = client.get_stats();
client.close().await?;
```

### Server Operations

```rust
use voltage_modbus::{ModbusTcpServer, ModbusTcpServerConfig, ModbusRegisterBank};
use std::sync::Arc;

// Create server
let config = ModbusTcpServerConfig {
    bind_address: "127.0.0.1:502".parse().unwrap(),
    max_connections: 50,
    request_timeout: Duration::from_secs(30),
    register_bank: Some(Arc::new(ModbusRegisterBank::new())),
};

let mut server = ModbusTcpServer::with_config(config)?;

// Server lifecycle
server.start().await?;
let stats = server.get_stats();
server.stop().await?;
```

### Register Bank Operations

```rust
use voltage_modbus::ModbusRegisterBank;

let bank = ModbusRegisterBank::new();

// Coil operations
bank.read_coils(address, count)?;
bank.write_single_coil(address, value)?;
bank.write_multiple_coils(address, &values)?;

// Register operations
bank.read_holding_registers(address, count)?;
bank.write_single_register(address, value)?;
bank.write_multiple_registers(address, &values)?;

// Statistics
let stats = bank.get_stats();
```

## 📊 Function Code Support

| Code | Function | Method | Status |
|------|----------|--------|--------|
| 0x01 | Read Coils | `read_coils()` | ✅ Full |
| 0x02 | Read Discrete Inputs | `read_discrete_inputs()` | ✅ Full |
| 0x03 | Read Holding Registers | `read_holding_registers()` | ✅ Full |
| 0x04 | Read Input Registers | `read_input_registers()` | ✅ Full |
| 0x05 | Write Single Coil | `write_single_coil()` | ✅ Full |
| 0x06 | Write Single Register | `write_single_register()` | ✅ Full |
| 0x0F | Write Multiple Coils | `write_multiple_coils()` | ✅ Full |
| 0x10 | Write Multiple Registers | `write_multiple_registers()` | ✅ Full |

## 🔧 Configuration Options

### Client Configuration
- **Address**: TCP socket address
- **Timeout**: Request timeout duration
- **Connection pooling**: Automatic connection management

### Server Configuration
- **Bind Address**: Server listening address
- **Max Connections**: Concurrent client limit
- **Request Timeout**: Per-request timeout
- **Register Bank**: Custom storage backend

### Register Bank Configuration
- **Coils**: Boolean values (default: 10,000)
- **Discrete Inputs**: Read-only boolean values (default: 10,000)
- **Holding Registers**: Read/write 16-bit values (default: 10,000)
- **Input Registers**: Read-only 16-bit values (default: 10,000)

## 📈 Performance Characteristics

### Benchmarks
- **Latency**: < 1ms (local network)
- **Throughput**: 1000+ requests/second
- **Concurrent Connections**: 50+ clients
- **Memory Usage**: < 10MB baseline
- **CPU Usage**: < 5% idle

### Optimization Features
- Async I/O with Tokio
- Zero-copy operations where possible
- Connection pooling
- Lock-free operations
- Configurable timeouts

## 🛡️ Error Handling

### Error Types
- **`ConnectionError`**: Network connectivity issues
- **`ProtocolError`**: Modbus protocol violations
- **`TimeoutError`**: Request timeout exceeded
- **`InvalidDataError`**: Invalid parameters or data
- **`ServerError`**: Server-side errors

### Error Recovery
- Automatic connection retry
- Graceful error propagation
- Detailed error context
- Logging integration

## 🧪 Testing and Examples

### Available Examples
- **`demo`**: Basic client operations
- **`server_demo`**: Complete server implementation
- **`full_function_test`**: All function codes test
- **`advanced_test`**: Performance and stress testing
- **`simple_test`**: Basic connectivity test

### Running Examples
```bash
# Start server
cargo run --bin server_demo

# Run client demo
cargo run --bin demo

# Run comprehensive tests
cargo run --bin full_function_test

# Performance testing
cargo run --bin advanced_test
```

## 📝 Documentation Generation

To regenerate this documentation:

```bash
# Generate API docs (no dependencies)
cargo doc --no-deps --all-features

# Open in browser
cargo doc --no-deps --all-features --open

# Generate with private items (no dependencies)
cargo doc --no-deps --all-features --document-private-items

# Generate with dependencies (if needed)
cargo doc --all-features
```

## 🤝 Contributing

For contributing to the API:

1. **Add documentation** to all public items
2. **Include examples** in doc comments
3. **Update this file** when adding new modules
4. **Run doc tests**: `cargo test --doc`
5. **Check doc warnings**: `cargo doc --all-features`

## 📞 Support

- **API Documentation**: https://docs.rs/voltage_modbus
- **Source Code**: https://github.com/voltage-llc/voltage_modbus
- **Issues**: https://github.com/voltage-llc/voltage_modbus/issues
- **Author**: Evan Liu <evan.liu@voltageenergy.com>

---

**Generated by Voltage Modbus v0.1.0 - Built with ❤️ by Evan Liu** 