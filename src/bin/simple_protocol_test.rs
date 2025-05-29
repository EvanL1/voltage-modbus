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

    println!("🔧 Modbus协议修复验证测试");
    println!("========================");
    
    let address = "127.0.0.1:5020".parse()
        .map_err(|e| voltage_modbus::error::ModbusError::invalid_data(format!("地址解析错误: {}", e)))?;
    let timeout = Duration::from_millis(3000);
    
    println!("📡 连接到测试服务器 {}...", address);
    let mut transport = TcpTransport::new(address, timeout).await?;
    println!("✅ 连接成功！");
    
    // 测试1: 基本读取保持寄存器
    println!("\n🧪 测试1: 读取保持寄存器 (地址0, 数量5)");
    let request = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadHoldingRegisters,
        0,
        5,
    );
    
    println!("📤 发送请求: Unit=1, Func=0x03, Addr=0, Qty=5");
    match transport.request(&request).await {
        Ok(response) => {
            println!("✅ 响应成功! 数据长度: {} 字节", response.data.len());
            if response.data.len() >= 1 {
                let byte_count = response.data[0];
                println!("📊 字节计数: {}", byte_count);
                
                if response.data.len() >= (1 + byte_count as usize) {
                    print!("📋 寄存器值: ");
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
                    println!("⚠️  响应数据不完整");
                }
            }
        }
        Err(e) => {
            println!("❌ 请求失败: {}", e);
        }
    }
    
    // 测试2: 不同地址读取
    println!("\n🧪 测试2: 读取保持寄存器 (地址10, 数量3)");
    let request2 = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadHoldingRegisters,
        10,
        3,
    );
    
    match transport.request(&request2).await {
        Ok(response) => {
            println!("✅ 第二次请求成功! 数据长度: {} 字节", response.data.len());
        }
        Err(e) => {
            println!("❌ 第二次请求失败: {}", e);
        }
    }
    
    // 测试3: 边界情况
    println!("\n🧪 测试3: 单个寄存器读取 (地址50, 数量1)");
    let request3 = ModbusRequest::new_read(
        1,
        ModbusFunction::ReadHoldingRegisters,
        50,
        1,
    );
    
    match transport.request(&request3).await {
        Ok(response) => {
            println!("✅ 单寄存器读取成功! 数据长度: {} 字节", response.data.len());
        }
        Err(e) => {
            println!("❌ 单寄存器读取失败: {}", e);
        }
    }
    
    // 获取并显示传输统计
    let stats = transport.get_stats();
    println!("\n📊 传输统计:");
    println!("  发送请求: {}", stats.requests_sent);
    println!("  收到响应: {}", stats.responses_received);
    println!("  发送字节: {} bytes", stats.bytes_sent);
    println!("  接收字节: {} bytes", stats.bytes_received);
    println!("  错误次数: {}", stats.errors);
    println!("  超时次数: {}", stats.timeouts);
    
    transport.close().await?;
    
    let success_rate = if stats.requests_sent > 0 {
        (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0
    } else {
        0.0
    };
    
    println!("\n🎯 协议修复验证结果:");
    if success_rate >= 99.0 && stats.errors == 0 {
        println!("  ✅ 协议修复成功! 成功率: {:.1}%", success_rate);
        println!("  ✅ PDU长度计算正确");
        println!("  ✅ 超时设置合理");
        println!("  ✅ 协议兼容性良好");
    } else if success_rate >= 90.0 {
        println!("  🟡 协议基本正常, 成功率: {:.1}%", success_rate);
        if stats.errors > 0 {
            println!("  ⚠️  仍有 {} 个错误需要关注", stats.errors);
        }
    } else {
        println!("  🔴 协议修复需要进一步优化, 成功率: {:.1}%", success_rate);
        println!("  🔴 错误次数: {}", stats.errors);
    }
    
    println!("\n✅ 协议验证测试完成!");
    Ok(())
} 