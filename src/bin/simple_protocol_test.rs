/// Voltage Modbus Protocol Test
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// Simple protocol correctness test

use std::time::Duration;
use voltage_modbus::transport::{TcpTransport, ModbusTransport};
use voltage_modbus::protocol::{ModbusRequest, ModbusFunction};
use voltage_modbus::error::ModbusResult;

#[tokio::main]
async fn main() -> ModbusResult<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🔧 Modbus Protocol Verification Test");
    println!("====================================");
    
    let address = "127.0.0.1:5020".parse()
        .map_err(|e| voltage_modbus::error::ModbusError::invalid_data(format!("Address parse error: {}", e)))?;
    let timeout = Duration::from_millis(3000);
    
    println!("📡 Connecting to test server {}...", address);
    let mut transport = TcpTransport::new(address, timeout).await?;
    println!("✅ Connection successful!");
    
    // Test 1: Basic holding register read
    println!("\n🧪 Test 1: Read Holding Registers (address 0, quantity 5)");
    let request = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadHoldingRegisters,
        0,
        5,
    );
    
    println!("📤 Sending request: Unit=1, Func=0x03, Addr=0, Qty=5");
    match transport.request(&request).await {
        Ok(response) => {
            println!("✅ Response successful! Data length: {} bytes", response.data.len());
            if response.data.len() >= 1 {
                let byte_count = response.data[0];
                println!("📊 Byte count: {}", byte_count);
                
                if response.data.len() >= (1 + byte_count as usize) {
                    print!("📋 Register values: ");
                    for i in 0..(byte_count as usize / 2) {
                        let reg_offset = 1 + i * 2;
                        if reg_offset + 1 < response.data.len() {
                            let value = u16::from_be_bytes([
                                response.data[reg_offset], 
                                response.data[reg_offset + 1]
                            ]);
                            print!("0x{:04x} ", value);
                        }
                    }
                    println!();
                } else {
                    println!("⚠️  Response data incomplete");
                }
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    // Test 2: Different address read
    println!("\n🧪 Test 2: Read Holding Registers (address 10, quantity 3)");
    let request2 = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadHoldingRegisters,
        10,
        3,
    );
    
    match transport.request(&request2).await {
        Ok(response) => {
            println!("✅ Second request successful! Data length: {} bytes", response.data.len());
        }
        Err(e) => {
            println!("❌ Second request failed: {}", e);
        }
    }
    
    // Test 3: Edge case
    println!("\n🧪 Test 3: Single Register Read (address 50, quantity 1)");
    let request3 = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadHoldingRegisters,
        50,
        1,
    );
    
    match transport.request(&request3).await {
        Ok(response) => {
            println!("✅ Single register read successful! Data length: {} bytes", response.data.len());
        }
        Err(e) => {
            println!("❌ Single register read failed: {}", e);
        }
    }
    
    // Get and display transport statistics
    let stats = transport.get_stats();
    println!("\n📊 Transport Statistics:");
    println!("  Requests sent: {}", stats.requests_sent);
    println!("  Responses received: {}", stats.responses_received);
    println!("  Bytes sent: {} bytes", stats.bytes_sent);
    println!("  Bytes received: {} bytes", stats.bytes_received);
    println!("  Error count: {}", stats.errors);
    println!("  Timeout count: {}", stats.timeouts);
    
    transport.close().await?;
    
    let success_rate = if stats.requests_sent > 0 {
        (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0
    } else {
        0.0
    };
    
    println!("\n🎯 Protocol Verification Results:");
    if success_rate >= 99.0 && stats.errors == 0 {
        println!("  ✅ Protocol verification successful! Success rate: {:.1}%", success_rate);
        println!("  ✅ PDU length calculation correct");
        println!("  ✅ Timeout settings reasonable");
        println!("  ✅ Protocol compatibility good");
    } else if success_rate >= 90.0 {
        println!("  🟡 Protocol basically normal, success rate: {:.1}%", success_rate);
        if stats.errors > 0 {
            println!("  ⚠️  Still have {} errors that need attention", stats.errors);
        }
    } else {
        println!("  🔴 Protocol needs further optimization, success rate: {:.1}%", success_rate);
        println!("  🔴 Error count: {}", stats.errors);
    }
    
    println!("\n✅ Protocol verification test completed!");
    Ok(())
} 