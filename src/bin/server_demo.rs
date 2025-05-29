/// Voltage Modbus Server Demo
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// Demonstrates complete Modbus TCP server functionality with all standard function code support

use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::time::{sleep, interval};
use log::{info, error};

use voltage_modbus::{
    ModbusTcpServer, ModbusTcpServerConfig, ModbusServer, 
    ModbusRegisterBank, RegisterBankStats
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Voltage Modbus Server Demo");
    println!("=============================");
    println!("Features:");
    println!("- Complete Modbus TCP protocol support");
    println!("- Support for all standard function codes (0x01-0x10)");
    println!("- High-concurrency client handling");
    println!("- Thread-safe register storage");
    println!("- Real-time statistics monitoring");
    println!("");

    // Create custom register storage
    let register_bank = Arc::new(ModbusRegisterBank::new());
    
    // Initialize test data
    info!("🔧 Initializing test data...");
    for i in 0..50 {
        register_bank.write_single_register(i, 0x1000 + i).unwrap();
        register_bank.write_single_coil(i, (i % 3) == 0).unwrap();
        register_bank.set_input_register(i, 0x2000 + i).unwrap();
        register_bank.set_discrete_input(i, (i % 2) == 0).unwrap();
    }

    // Configure server
    let config = ModbusTcpServerConfig {
        bind_address: "127.0.0.1:5020".parse().unwrap(),
        max_connections: 50,
        request_timeout: Duration::from_secs(30),
        register_bank: Some(register_bank.clone()),
    };

    // Create and start server
    let mut server = ModbusTcpServer::with_config(config)?;
    
    info!("🚀 Starting Modbus TCP server...");
    server.start().await?;
    
    info!("✅ Server started successfully!");
    info!("📍 Listening on: 127.0.0.1:5020");
    info!("🔗 Supported function codes:");
    info!("   - 0x01: Read Coils");
    info!("   - 0x02: Read Discrete Inputs");
    info!("   - 0x03: Read Holding Registers");
    info!("   - 0x04: Read Input Registers");
    info!("   - 0x05: Write Single Coil");
    info!("   - 0x06: Write Single Register");
    info!("   - 0x0F: Write Multiple Coils");
    info!("   - 0x10: Write Multiple Registers");
    
    // Start statistics monitoring task
    let register_bank_stats = register_bank.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            let stats = register_bank_stats.get_stats();
            info!("📊 Register storage statistics:");
            info!("   Coils count: {}", stats.coils_count);
            info!("   Discrete inputs count: {}", stats.discrete_inputs_count);
            info!("   Holding registers count: {}", stats.holding_registers_count);
            info!("   Input registers count: {}", stats.input_registers_count);
        }
    });

    // Start data simulation task
    let register_bank_sim = register_bank.clone();
    tokio::spawn(async move {
        let mut counter = 0u16;
        let mut interval = interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Simulate dynamic data changes
            for i in 50..60 {
                let _ = register_bank_sim.set_input_register(i, 0x3000 + counter + i);
                let _ = register_bank_sim.set_discrete_input(i, (counter + i) % 4 == 0);
            }
            
            counter = counter.wrapping_add(1);
            info!("🔄 Data simulation update: counter = {}", counter);
        }
    });

    println!("\n📋 Server running...");
    println!("💡 Testing suggestions:");
    println!("   - Use client programs to connect to 127.0.0.1:5020");
    println!("   - Test reading addresses 0-49 for various data types");
    println!("   - Test write functions and data persistence");
    println!("   - Addresses 50-59 have dynamically changing simulated data");
    println!("   - Press Ctrl+C to stop the server");
    println!("");

    // Wait for interrupt signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("🛑 Received interrupt signal, stopping server...");
        }
        Err(err) => {
            error!("❌ Failed to listen for interrupt signal: {}", err);
        }
    }

    // Stop server
    server.stop().await?;
    
    // Display final statistics
    let final_stats = server.get_stats();
    info!("📊 Final server statistics:");
    info!("   Total connections: {}", final_stats.connections_count);
    info!("   Total requests: {}", final_stats.total_requests);
    info!("   Successful requests: {}", final_stats.successful_requests);
    info!("   Failed requests: {}", final_stats.failed_requests);
    info!("   Bytes received: {} bytes", final_stats.bytes_received);
    info!("   Bytes sent: {} bytes", final_stats.bytes_sent);
    info!("   Uptime: {} seconds", final_stats.uptime_seconds);
    
    println!("\n✅ Server stopped safely");
    println!("👋 Thank you for using Voltage Modbus Server by Evan Liu!");

    Ok(())
} 