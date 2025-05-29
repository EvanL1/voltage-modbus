/// Voltage Modbus Simple Test
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// Basic functionality test for the voltage_modbus library

use std::time::Duration;
use tokio;
use voltage_modbus::transport::{TcpTransport, ModbusTransport};
use voltage_modbus::protocol::{ModbusRequest, ModbusFunction};
use voltage_modbus::error::ModbusResult;

#[tokio::main]
async fn main() -> ModbusResult<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Voltage Modbus Library Test");
    println!("==============================");

    // Test basic TCP client functionality
    test_tcp_client().await?;
    
    println!("\n✅ All tests completed successfully!");
    Ok(())
}

async fn test_tcp_client() -> ModbusResult<()> {
    println!("📡 Testing TCP Client...");
    
    let address = "127.0.0.1:5020".parse().expect("Invalid address");
    let timeout = Duration::from_secs(5);
    
    // Try to create a client
    println!("  Creating TCP client for {}...", address);
    match TcpTransport::new(address, timeout).await {
        Ok(mut transport) => {
            println!("  ✅ TCP client created successfully");
            
            // Test connection
            if transport.is_connected() {
                println!("  ✅ Connection established");
                
                // Test basic read request
                let request = ModbusRequest::new_read(
                    1, // slave_id
                    ModbusFunction::ReadHoldingRegisters,
                    0, // address
                    10, // quantity - read 10 registers
                );
                
                println!("  📤 Sending read holding registers request...");
                match transport.request(&request).await {
                    Ok(response) => {
                        println!("  ✅ Response received: {} bytes", response.data.len());
                        println!("  📊 Data: {:?}", &response.data[..std::cmp::min(10, response.data.len())]);
                    }
                    Err(e) => {
                        println!("  ⚠️  Request failed: {}", e);
                        println!("  (This is expected if no server is running)");
                    }
                }
                
                // Get stats
                let stats = transport.get_stats();
                println!("  📊 Transport Stats:");
                println!("    Requests sent: {}", stats.requests_sent);
                println!("    Responses received: {}", stats.responses_received);
                println!("    Errors: {}", stats.errors);
                println!("    Timeouts: {}", stats.timeouts);
                
                // Close connection
                transport.close().await?;
                println!("  🔌 Connection closed");
            } else {
                println!("  ⚠️  Connection not established");
            }
        }
        Err(e) => {
            println!("  ⚠️  Failed to create TCP client: {}", e);
            println!("  (This is expected if no server is running)");
        }
    }
    
    println!("📡 TCP Client test completed\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_creation() {
        let request = ModbusRequest::new_read(
            1,
            ModbusFunction::ReadHoldingRegisters,
            100,
            5
        );
        
        assert_eq!(request.slave_id, 1);
        assert_eq!(request.function, ModbusFunction::ReadHoldingRegisters);
        assert_eq!(request.address, 100);
        assert_eq!(request.quantity, 5);
    }
} 