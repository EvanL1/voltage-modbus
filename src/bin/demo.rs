/// Voltage Modbus Demo
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// This program demonstrates basic usage of the voltage_modbus library.

use std::time::Duration;
use tokio::time::sleep;
use voltage_modbus::{ModbusClient, ModbusTcpClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Voltage Modbus Demo");
    println!("=====================");
    
    let server_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:502".to_string());
    
    println!("Connecting to Modbus server at {}...", server_address);
    
    let timeout = Duration::from_secs(5);
    
    // Parse address
    let address: std::net::SocketAddr = server_address.parse()
        .map_err(|e| format!("Invalid server address: {}", e))?;
    
    let mut client = match ModbusTcpClient::with_timeout(&server_address, timeout).await {
        Ok(client) => {
            println!("✅ Connected successfully!");
            client
        },
        Err(e) => {
            eprintln!("❌ Failed to connect: {}", e);
            eprintln!("Make sure a Modbus server is running on {}", server_address);
            return Ok(());
        }
    };
    
    let slave_id = 1;
    
    println!("\n📖 Testing read operations...");
    
    // Test reading holding registers
    match client.read_holding_registers(slave_id, 100, 5).await {
        Ok(values) => {
            println!("✅ Read holding registers 100-104: {:?}", values);
            for (i, value) in values.iter().enumerate() {
                println!("   Register {}: 0x{:04X} ({})", 100 + i, value, value);
            }
        },
        Err(e) => println!("❌ Failed to read holding registers: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test reading input registers
    match client.read_input_registers(slave_id, 200, 3).await {
        Ok(values) => {
            println!("✅ Read input registers 200-202: {:?}", values);
        },
        Err(e) => println!("❌ Failed to read input registers: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test reading coils
    match client.read_coils(slave_id, 0, 8).await {
        Ok(values) => {
            println!("✅ Read coils 0-7: {:?}", values);
            for (i, &coil) in values.iter().enumerate() {
                println!("   Coil {}: {}", i, if coil { "ON" } else { "OFF" });
            }
        },
        Err(e) => println!("❌ Failed to read coils: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test reading discrete inputs
    match client.read_discrete_inputs(slave_id, 100, 4).await {
        Ok(values) => {
            println!("✅ Read discrete inputs 100-103: {:?}", values);
        },
        Err(e) => println!("❌ Failed to read discrete inputs: {}", e),
    }
    
    println!("\n✏️  Testing write operations...");
    
    // Test writing single register
    let test_value = 0x1234;
    match client.write_single_register(slave_id, 300, test_value).await {
        Ok(_) => {
            println!("✅ Wrote single register 300 = 0x{:04X}", test_value);
            
            // Read it back to verify
            sleep(Duration::from_millis(50)).await;
            match client.read_holding_registers(slave_id, 300, 1).await {
                Ok(values) if !values.is_empty() => {
                    if values[0] == test_value {
                        println!("✅ Verified: register 300 = 0x{:04X}", values[0]);
                    } else {
                        println!("⚠️  Value mismatch: expected 0x{:04X}, got 0x{:04X}", test_value, values[0]);
                    }
                },
                Ok(_) => println!("⚠️  Read back empty result"),
                Err(e) => println!("❌ Failed to read back register: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to write single register: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test writing multiple registers
    let test_values = vec![0x1111, 0x2222, 0x3333];
    match client.write_multiple_registers(slave_id, 400, &test_values).await {
        Ok(_) => {
            println!("✅ Wrote multiple registers 400-402: {:?}", test_values);
            
            // Read them back to verify
            sleep(Duration::from_millis(50)).await;
            match client.read_holding_registers(slave_id, 400, test_values.len() as u16).await {
                Ok(values) => {
                    if values == test_values {
                        println!("✅ Verified: registers 400-402 = {:?}", values);
                    } else {
                        println!("⚠️  Value mismatch: expected {:?}, got {:?}", test_values, values);
                    }
                },
                Err(e) => println!("❌ Failed to read back registers: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to write multiple registers: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test writing single coil
    match client.write_single_coil(slave_id, 10, true).await {
        Ok(_) => {
            println!("✅ Wrote single coil 10 = ON");
            
            // Read it back to verify
            sleep(Duration::from_millis(50)).await;
            match client.read_coils(slave_id, 10, 1).await {
                Ok(values) if !values.is_empty() => {
                    if values[0] {
                        println!("✅ Verified: coil 10 = ON");
                    } else {
                        println!("⚠️  Value mismatch: expected ON, got OFF");
                    }
                },
                Ok(_) => println!("⚠️  Read back empty result"),
                Err(e) => println!("❌ Failed to read back coil: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to write single coil: {}", e),
    }
    
    sleep(Duration::from_millis(100)).await;
    
    // Test writing multiple coils
    let test_coils = vec![true, false, true, false, true];
    match client.write_multiple_coils(slave_id, 20, &test_coils).await {
        Ok(_) => {
            println!("✅ Wrote multiple coils 20-24: {:?}", test_coils);
            
            // Read them back to verify
            sleep(Duration::from_millis(50)).await;
            match client.read_coils(slave_id, 20, test_coils.len() as u16).await {
                Ok(values) => {
                    let trimmed_values: Vec<bool> = values.into_iter().take(test_coils.len()).collect();
                    if trimmed_values == test_coils {
                        println!("✅ Verified: coils 20-24 = {:?}", trimmed_values);
                    } else {
                        println!("⚠️  Value mismatch: expected {:?}, got {:?}", test_coils, trimmed_values);
                    }
                },
                Err(e) => println!("❌ Failed to read back coils: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to write multiple coils: {}", e),
    }
    
    // Show connection statistics
    let stats = client.get_stats();
    println!("\n📊 Connection Statistics:");
    println!("   Requests sent: {}", stats.requests_sent);
    println!("   Responses received: {}", stats.responses_received);
    println!("   Errors: {}", stats.errors);
    println!("   Timeouts: {}", stats.timeouts);
    println!("   Bytes sent: {}", stats.bytes_sent);
    println!("   Bytes received: {}", stats.bytes_received);
    
    if stats.requests_sent > 0 {
        let success_rate = (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0;
        println!("   Success rate: {:.1}%", success_rate);
    }
    
    // Close connection
    if let Err(e) = client.close().await {
        eprintln!("⚠️  Error closing connection: {}", e);
    } else {
        println!("\n✅ Connection closed successfully");
    }
    
    println!("\n🎉 Demo completed!");
    println!("👋 Thank you for using Voltage Modbus by Evan Liu!");
    
    Ok(())
} 