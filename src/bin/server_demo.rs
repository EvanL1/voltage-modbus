/// Voltage Modbus Server Demo
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// 演示完整的Modbus TCP服务器功能，包括所有标准功能码支持

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
    println!("功能特点:");
    println!("- 完整Modbus TCP协议支持");
    println!("- 支持所有标准功能码 (0x01-0x10)");
    println!("- 高并发客户端处理");
    println!("- 线程安全的寄存器存储");
    println!("- 实时统计监控");
    println!("");

    // 创建自定义寄存器存储
    let register_bank = Arc::new(ModbusRegisterBank::new());
    
    // 初始化一些测试数据
    info!("🔧 初始化测试数据...");
    for i in 0..50 {
        register_bank.write_single_register(i, 0x1000 + i).unwrap();
        register_bank.write_single_coil(i, (i % 3) == 0).unwrap();
        register_bank.set_input_register(i, 0x2000 + i).unwrap();
        register_bank.set_discrete_input(i, (i % 2) == 0).unwrap();
    }

    // 配置服务器
    let config = ModbusTcpServerConfig {
        bind_address: "127.0.0.1:5020".parse().unwrap(),
        max_connections: 50,
        request_timeout: Duration::from_secs(30),
        register_bank: Some(register_bank.clone()),
    };

    // 创建并启动服务器
    let mut server = ModbusTcpServer::with_config(config)?;
    
    info!("🚀 启动Modbus TCP服务器...");
    server.start().await?;
    
    info!("✅ 服务器启动成功!");
    info!("📍 监听地址: 127.0.0.1:5020");
    info!("🔗 支持的功能码:");
    info!("   - 0x01: 读取线圈 (Read Coils)");
    info!("   - 0x02: 读取离散输入 (Read Discrete Inputs)");
    info!("   - 0x03: 读取保持寄存器 (Read Holding Registers)");
    info!("   - 0x04: 读取输入寄存器 (Read Input Registers)");
    info!("   - 0x05: 写入单个线圈 (Write Single Coil)");
    info!("   - 0x06: 写入单个寄存器 (Write Single Register)");
    info!("   - 0x0F: 写入多个线圈 (Write Multiple Coils)");
    info!("   - 0x10: 写入多个寄存器 (Write Multiple Registers)");
    
    // 启动统计监控任务
    let register_bank_stats = register_bank.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            let stats = register_bank_stats.get_stats();
            info!("📊 寄存器存储统计:");
            info!("   线圈数量: {}", stats.coils_count);
            info!("   离散输入数量: {}", stats.discrete_inputs_count);
            info!("   保持寄存器数量: {}", stats.holding_registers_count);
            info!("   输入寄存器数量: {}", stats.input_registers_count);
        }
    });

    // 启动数据模拟任务
    let register_bank_sim = register_bank.clone();
    tokio::spawn(async move {
        let mut counter = 0u16;
        let mut interval = interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // 模拟动态数据变化
            for i in 50..60 {
                let _ = register_bank_sim.set_input_register(i, 0x3000 + counter + i);
                let _ = register_bank_sim.set_discrete_input(i, (counter + i) % 4 == 0);
            }
            
            counter = counter.wrapping_add(1);
            info!("🔄 数据模拟更新: 计数器 = {}", counter);
        }
    });

    println!("\n📋 服务器运行中...");
    println!("💡 测试建议:");
    println!("   - 使用客户端程序连接到 127.0.0.1:5020");
    println!("   - 测试读取地址 0-49 的各种数据类型");
    println!("   - 测试写入功能和数据持久性");
    println!("   - 地址 50-59 有动态变化的模拟数据");
    println!("   - 按 Ctrl+C 停止服务器");
    println!("");

    // 等待中断信号
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("🛑 收到中断信号，正在停止服务器...");
        }
        Err(err) => {
            error!("❌ 监听中断信号失败: {}", err);
        }
    }

    // 停止服务器
    server.stop().await?;
    
    // 显示最终统计
    let final_stats = server.get_stats();
    info!("📊 服务器最终统计:");
    info!("   连接总数: {}", final_stats.connections_count);
    info!("   总请求数: {}", final_stats.total_requests);
    info!("   成功请求: {}", final_stats.successful_requests);
    info!("   失败请求: {}", final_stats.failed_requests);
    info!("   接收字节: {} bytes", final_stats.bytes_received);
    info!("   发送字节: {} bytes", final_stats.bytes_sent);
    info!("   运行时间: {} 秒", final_stats.uptime_seconds);
    
    println!("\n✅ 服务器已安全停止");
    println!("👋 感谢使用 Voltage Modbus Server by Evan Liu!");

    Ok(())
} 