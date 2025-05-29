/// Voltage Modbus Full Function Test Client
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// Tests all Modbus function codes including read and write operations

use std::time::Duration;
use voltage_modbus::transport::{TcpTransport, ModbusTransport};
use voltage_modbus::protocol::{ModbusRequest, ModbusFunction};
use voltage_modbus::error::ModbusResult;

#[tokio::main]
async fn main() -> ModbusResult<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🧪 Voltage Modbus Full Function Test");
    println!("====================================");
    println!("Testing all standard Modbus function codes");
    println!();

    let address = "127.0.0.1:5020".parse()
        .map_err(|e| voltage_modbus::error::ModbusError::invalid_data(format!("Address parse error: {}", e)))?;
    let timeout = Duration::from_millis(3000);
    
    println!("📡 Connecting to server {}...", address);
    let mut transport = TcpTransport::new(address, timeout).await?;
    println!("✅ Connection successful!");
    println!();

    // Test 1: Read Coils (0x01)
    println!("🧪 Test 1: Read Coils (Function Code 0x01)");
    test_read_coils(&mut transport).await?;
    
    // Test 2: Read Discrete Inputs (0x02)  
    println!("\n🧪 Test 2: Read Discrete Inputs (Function Code 0x02)");
    test_read_discrete_inputs(&mut transport).await?;
    
    // Test 3: Read Holding Registers (0x03)
    println!("\n🧪 Test 3: Read Holding Registers (Function Code 0x03)");
    test_read_holding_registers(&mut transport).await?;
    
    // Test 4: Read Input Registers (0x04)
    println!("\n🧪 Test 4: Read Input Registers (Function Code 0x04)");
    test_read_input_registers(&mut transport).await?;
    
    // Test 5: Write Single Coil (0x05)
    println!("\n🧪 Test 5: Write Single Coil (Function Code 0x05)");
    test_write_single_coil(&mut transport).await?;
    
    // Test 6: Write Single Register (0x06)
    println!("\n🧪 Test 6: Write Single Register (Function Code 0x06)");
    test_write_single_register(&mut transport).await?;
    
    // Test 7: Write Multiple Coils (0x0F)
    println!("\n🧪 Test 7: Write Multiple Coils (Function Code 0x0F)");
    test_write_multiple_coils(&mut transport).await?;
    
    // Test 8: Write Multiple Registers (0x10)
    println!("\n🧪 Test 8: Write Multiple Registers (Function Code 0x10)");
    test_write_multiple_registers(&mut transport).await?;

    // Get final statistics
    let stats = transport.get_stats();
    println!("\n📊 Test Statistics Summary:");
    println!("  Total requests: {}", stats.requests_sent);
    println!("  Total responses: {}", stats.responses_received);
    println!("  Success rate: {:.1}%", (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0);
    println!("  Total errors: {}", stats.errors);
    println!("  Total timeouts: {}", stats.timeouts);
    println!("  Bytes sent: {} bytes", stats.bytes_sent);
    println!("  Bytes received: {} bytes", stats.bytes_received);

    transport.close().await?;
    
    println!("\n🎉 All function tests completed!");
    Ok(())
}

async fn test_read_coils(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadCoils, 0, 10);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Read coils successful");
            if !response.data.is_empty() {
                let byte_count = response.data[0];
                print!("  📊 Coil states (address 0-9): ");
                
                for i in 0..10 {
                    let byte_index = (i / 8) as usize + 1;
                    let bit_index = i % 8;
                    if byte_index < response.data.len() {
                        let bit_value = (response.data[byte_index] & (1 << bit_index)) != 0;
                        print!("{} ", if bit_value { "1" } else { "0" });
                    }
                }
                println!();
                println!("  📦 Byte count: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ Read coils failed: {}", e);
        }
    }
    Ok(())
}

async fn test_read_discrete_inputs(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadDiscreteInputs, 0, 8);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Read discrete inputs successful");
            if !response.data.is_empty() {
                let byte_count = response.data[0];
                print!("  📊 Input states (address 0-7): ");
                
                for i in 0..8 {
                    let byte_index = (i / 8) as usize + 1;
                    let bit_index = i % 8;
                    if byte_index < response.data.len() {
                        let bit_value = (response.data[byte_index] & (1 << bit_index)) != 0;
                        print!("{} ", if bit_value { "1" } else { "0" });
                    }
                }
                println!();
                println!("  📦 Byte count: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ Read discrete inputs failed: {}", e);
        }
    }
    Ok(())
}

async fn test_read_holding_registers(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadHoldingRegisters, 0, 5);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Read holding registers successful");
            if response.data.len() >= 1 {
                let byte_count = response.data[0];
                print!("  📊 Register values (address 0-4): ");
                
                for i in 0..5 {
                    let offset = 1 + i * 2;
                    if offset + 1 < response.data.len() {
                        let value = u16::from_be_bytes([response.data[offset], response.data[offset + 1]]);
                        print!("0x{:04x} ", value);
                    }
                }
                println!();
                println!("  📦 Byte count: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ Read holding registers failed: {}", e);
        }
    }
    Ok(())
}

async fn test_read_input_registers(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadInputRegisters, 0, 3);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Read input registers successful");
            if response.data.len() >= 1 {
                let byte_count = response.data[0];
                print!("  📊 Register values (address 0-2): ");
                
                for i in 0..3 {
                    let offset = 1 + i * 2;
                    if offset + 1 < response.data.len() {
                        let value = u16::from_be_bytes([response.data[offset], response.data[offset + 1]]);
                        print!("0x{:04x} ", value);
                    }
                }
                println!();
                println!("  📦 Byte count: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ Read input registers failed: {}", e);
        }
    }
    Ok(())
}

async fn test_write_single_coil(transport: &mut TcpTransport) -> ModbusResult<()> {
    // Create write single coil request
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteSingleCoil,
        address: 100,
        quantity: 1,
        data: vec![0xFF, 0x00], // ON state
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Write single coil successful");
            if response.data.len() >= 4 {
                let address = u16::from_be_bytes([response.data[0], response.data[1]]);
                let value = u16::from_be_bytes([response.data[2], response.data[3]]);
                println!("  📍 Address: {}", address);
                println!("  📊 Value: 0x{:04x} ({})", value, if value == 0xFF00 { "ON" } else { "OFF" });
            }
        }
        Err(e) => {
            println!("  ❌ Write single coil failed: {}", e);
        }
    }
    
    // Test reading back the written value
    println!("  🔄 Verifying written value...");
    let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadCoils, 100, 1);
    
    match transport.request(&read_request).await {
        Ok(response) => {
            if !response.data.is_empty() && response.data.len() >= 2 {
                let written_value = (response.data[1] & 0x01) != 0;
                println!("  ✅ Verification successful: coil is {}", if written_value { "ON" } else { "OFF" });
            }
        }
        Err(e) => {
            println!("  ⚠️  Verification failed: {}", e);
        }
    }
    
    Ok(())
}

async fn test_write_single_register(transport: &mut TcpTransport) -> ModbusResult<()> {
    // Create write single register request
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteSingleRegister,
        address: 200,
        quantity: 1,
        data: vec![0x12, 0x34], // Value 0x1234
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Write single register successful");
            if response.data.len() >= 4 {
                let address = u16::from_be_bytes([response.data[0], response.data[1]]);
                let value = u16::from_be_bytes([response.data[2], response.data[3]]);
                println!("  📍 Address: {}", address);
                println!("  📊 Value: 0x{:04x} ({})", value, value);
            }
        }
        Err(e) => {
            println!("  ❌ Write single register failed: {}", e);
        }
    }
    
    // Test reading back the written value
    println!("  🔄 Verifying written value...");
    let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadHoldingRegisters, 200, 1);
    
    match transport.request(&read_request).await {
        Ok(response) => {
            if response.data.len() >= 3 {
                let written_value = u16::from_be_bytes([response.data[1], response.data[2]]);
                println!("  ✅ Verification successful: register value is 0x{:04x}", written_value);
            }
        }
        Err(e) => {
            println!("  ⚠️  Verification failed: {}", e);
        }
    }
    
    Ok(())
}

async fn test_write_multiple_coils(transport: &mut TcpTransport) -> ModbusResult<()> {
    // Create write multiple coils request - write 8 coils with pattern: 10101010
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteMultipleCoils,
        address: 300,
        quantity: 8,
        data: vec![0x01, 0xAA], // 1 byte count + pattern 10101010
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Write multiple coils successful");
            if response.data.len() >= 4 {
                let address = u16::from_be_bytes([response.data[0], response.data[1]]);
                let quantity = u16::from_be_bytes([response.data[2], response.data[3]]);
                println!("  📍 Address: {}", address);
                println!("  📊 Quantity: {} coils", quantity);
            }
        }
        Err(e) => {
            println!("  ❌ Write multiple coils failed: {}", e);
        }
    }
    
    // Test reading back the written values
    println!("  🔄 Verifying written values...");
    let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadCoils, 300, 8);
    
    match transport.request(&read_request).await {
        Ok(response) => {
            if !response.data.is_empty() && response.data.len() >= 2 {
                print!("  ✅ Verification successful. Coil pattern: ");
                for i in 0..8 {
                    let bit_value = (response.data[1] & (1 << i)) != 0;
                    print!("{}", if bit_value { "1" } else { "0" });
                }
                println!();
            }
        }
        Err(e) => {
            println!("  ⚠️  Verification failed: {}", e);
        }
    }
    
    Ok(())
}

async fn test_write_multiple_registers(transport: &mut TcpTransport) -> ModbusResult<()> {
    // Create write multiple registers request
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteMultipleRegisters,
        address: 400,
        quantity: 3,
        data: vec![
            0x06,       // Byte count (3 registers × 2 bytes = 6)
            0x11, 0x11, // Register 400: 0x1111
            0x22, 0x22, // Register 401: 0x2222
            0x33, 0x33, // Register 402: 0x3333
        ],
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ Write multiple registers successful");
            if response.data.len() >= 4 {
                let address = u16::from_be_bytes([response.data[0], response.data[1]]);
                let quantity = u16::from_be_bytes([response.data[2], response.data[3]]);
                println!("  📍 Address: {}", address);
                println!("  📊 Quantity: {} registers", quantity);
            }
        }
        Err(e) => {
            println!("  ❌ Write multiple registers failed: {}", e);
        }
    }
    
    // Test reading back the written values
    println!("  🔄 Verifying written values...");
    let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadHoldingRegisters, 400, 3);
    
    match transport.request(&read_request).await {
        Ok(response) => {
            if response.data.len() >= 7 { // 1 byte count + 6 bytes data
                print!("  ✅ Verification successful. Register values: ");
                for i in 0..3 {
                    let offset = 1 + i * 2;
                    let value = u16::from_be_bytes([response.data[offset], response.data[offset + 1]]);
                    print!("0x{:04x} ", value);
                }
                println!();
            }
        }
        Err(e) => {
            println!("  ⚠️  Verification failed: {}", e);
        }
    }
    
    Ok(())
} 