/// Voltage Modbus Full Function Test Client
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// 测试所有Modbus功能码，包括读写操作

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
    println!("测试所有标准Modbus功能码");
    println!();

    let address = "127.0.0.1:5020".parse()
        .map_err(|e| voltage_modbus::error::ModbusError::invalid_data(format!("地址解析错误: {}", e)))?;
    let timeout = Duration::from_millis(3000);
    
    println!("📡 连接到服务器 {}...", address);
    let mut transport = TcpTransport::new(address, timeout).await?;
    println!("✅ 连接成功!");
    println!();

    // 测试1: 读取线圈 (0x01)
    println!("🧪 测试1: 读取线圈 (Function Code 0x01)");
    test_read_coils(&mut transport).await?;
    
    // 测试2: 读取离散输入 (0x02)  
    println!("\n🧪 测试2: 读取离散输入 (Function Code 0x02)");
    test_read_discrete_inputs(&mut transport).await?;
    
    // 测试3: 读取保持寄存器 (0x03)
    println!("\n🧪 测试3: 读取保持寄存器 (Function Code 0x03)");
    test_read_holding_registers(&mut transport).await?;
    
    // 测试4: 读取输入寄存器 (0x04)
    println!("\n🧪 测试4: 读取输入寄存器 (Function Code 0x04)");
    test_read_input_registers(&mut transport).await?;
    
    // 测试5: 写入单个线圈 (0x05)
    println!("\n🧪 测试5: 写入单个线圈 (Function Code 0x05)");
    test_write_single_coil(&mut transport).await?;
    
    // 测试6: 写入单个寄存器 (0x06)
    println!("\n🧪 测试6: 写入单个寄存器 (Function Code 0x06)");
    test_write_single_register(&mut transport).await?;
    
    // 测试7: 写入多个线圈 (0x0F)
    println!("\n🧪 测试7: 写入多个线圈 (Function Code 0x0F)");
    test_write_multiple_coils(&mut transport).await?;
    
    // 测试8: 写入多个寄存器 (0x10)
    println!("\n🧪 测试8: 写入多个寄存器 (Function Code 0x10)");
    test_write_multiple_registers(&mut transport).await?;

    // 获取最终统计
    let stats = transport.get_stats();
    println!("\n📊 测试统计总结:");
    println!("  总请求数: {}", stats.requests_sent);
    println!("  总响应数: {}", stats.responses_received);
    println!("  成功率: {:.1}%", (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0);
    println!("  总错误数: {}", stats.errors);
    println!("  总超时数: {}", stats.timeouts);
    println!("  发送字节: {} bytes", stats.bytes_sent);
    println!("  接收字节: {} bytes", stats.bytes_received);

    transport.close().await?;
    
    println!("\n🎉 所有功能测试完成!");
    Ok(())
}

async fn test_read_coils(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadCoils, 0, 10);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 读取线圈成功");
            if !response.data.is_empty() {
                let byte_count = response.data[0];
                print!("  📊 线圈状态 (地址0-9): ");
                
                for i in 0..10 {
                    let byte_index = (i / 8) as usize + 1;
                    let bit_index = i % 8;
                    if byte_index < response.data.len() {
                        let bit_value = (response.data[byte_index] & (1 << bit_index)) != 0;
                        print!("{} ", if bit_value { "1" } else { "0" });
                    }
                }
                println!();
                println!("  📦 字节数: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ 读取线圈失败: {}", e);
        }
    }
    Ok(())
}

async fn test_read_discrete_inputs(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadDiscreteInputs, 0, 8);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 读取离散输入成功");
            if !response.data.is_empty() {
                let byte_count = response.data[0];
                print!("  📊 输入状态 (地址0-7): ");
                
                for i in 0..8 {
                    let byte_index = (i / 8) as usize + 1;
                    let bit_index = i % 8;
                    if byte_index < response.data.len() {
                        let bit_value = (response.data[byte_index] & (1 << bit_index)) != 0;
                        print!("{} ", if bit_value { "1" } else { "0" });
                    }
                }
                println!();
                println!("  📦 字节数: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ 读取离散输入失败: {}", e);
        }
    }
    Ok(())
}

async fn test_read_holding_registers(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadHoldingRegisters, 0, 5);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 读取保持寄存器成功");
            if response.data.len() >= 1 {
                let byte_count = response.data[0];
                print!("  📊 寄存器值 (地址0-4): ");
                
                for i in 0..5 {
                    let offset = 1 + i * 2;
                    if offset + 1 < response.data.len() {
                        let value = u16::from_be_bytes([response.data[offset], response.data[offset + 1]]);
                        print!("0x{:04x} ", value);
                    }
                }
                println!();
                println!("  📦 字节数: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ 读取保持寄存器失败: {}", e);
        }
    }
    Ok(())
}

async fn test_read_input_registers(transport: &mut TcpTransport) -> ModbusResult<()> {
    let request = ModbusRequest::new_read(1, ModbusFunction::ReadInputRegisters, 0, 3);
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 读取输入寄存器成功");
            if response.data.len() >= 1 {
                let byte_count = response.data[0];
                print!("  📊 寄存器值 (地址0-2): ");
                
                for i in 0..3 {
                    let offset = 1 + i * 2;
                    if offset + 1 < response.data.len() {
                        let value = u16::from_be_bytes([response.data[offset], response.data[offset + 1]]);
                        print!("0x{:04x} ", value);
                    }
                }
                println!();
                println!("  📦 字节数: {}", byte_count);
            }
        }
        Err(e) => {
            println!("  ❌ 读取输入寄存器失败: {}", e);
        }
    }
    Ok(())
}

async fn test_write_single_coil(transport: &mut TcpTransport) -> ModbusResult<()> {
    // 创建写入单个线圈的请求
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteSingleCoil,
        address: 100,
        quantity: 1,
        data: vec![1], // true
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 写入单个线圈成功");
            println!("  📝 写入地址100, 值: ON");
            println!("  📦 响应数据长度: {} bytes", response.data.len());
            
            // 验证写入 - 读取刚写入的线圈
            let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadCoils, 100, 1);
            match transport.request(&read_request).await {
                Ok(read_response) => {
                    if read_response.data.len() >= 2 {
                        let bit_value = (read_response.data[1] & 0x01) != 0;
                        println!("  🔍 验证读取: {} ({})", if bit_value { "ON" } else { "OFF" }, if bit_value { "✅" } else { "❌" });
                    }
                }
                Err(e) => {
                    println!("  ⚠️  验证读取失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 写入单个线圈失败: {}", e);
        }
    }
    Ok(())
}

async fn test_write_single_register(transport: &mut TcpTransport) -> ModbusResult<()> {
    // 创建写入单个寄存器的请求
    let test_value: u16 = 0xABCD;
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteSingleRegister,
        address: 200,
        quantity: 1,
        data: test_value.to_be_bytes().to_vec(),
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 写入单个寄存器成功");
            println!("  📝 写入地址200, 值: 0x{:04X}", test_value);
            println!("  📦 响应数据长度: {} bytes", response.data.len());
            
            // 验证写入 - 读取刚写入的寄存器
            let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadHoldingRegisters, 200, 1);
            match transport.request(&read_request).await {
                Ok(read_response) => {
                    if read_response.data.len() >= 3 {
                        let value = u16::from_be_bytes([read_response.data[1], read_response.data[2]]);
                        println!("  🔍 验证读取: 0x{:04X} ({})", value, if value == test_value { "✅" } else { "❌" });
                    }
                }
                Err(e) => {
                    println!("  ⚠️  验证读取失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 写入单个寄存器失败: {}", e);
        }
    }
    Ok(())
}

async fn test_write_multiple_coils(transport: &mut TcpTransport) -> ModbusResult<()> {
    // 创建写入多个线圈的请求 - 写入8个线圈的模式: 10101010
    let coil_values = vec![true, false, true, false, true, false, true, false];
    let mut data = Vec::new();
    
    // 将布尔值打包成字节
    let mut byte_value = 0u8;
    for (i, &coil) in coil_values.iter().enumerate() {
        if coil {
            byte_value |= 1 << i;
        }
    }
    data.push(1); // 字节数
    data.push(byte_value);
    
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteMultipleCoils,
        address: 300,
        quantity: 8,
        data,
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 写入多个线圈成功");
            println!("  📝 写入地址300-307, 模式: 10101010");
            println!("  📦 响应数据长度: {} bytes", response.data.len());
            
            // 验证写入 - 读取刚写入的线圈
            let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadCoils, 300, 8);
            match transport.request(&read_request).await {
                Ok(read_response) => {
                    if read_response.data.len() >= 2 {
                        print!("  🔍 验证读取: ");
                        for i in 0..8 {
                            let bit_value = (read_response.data[1] & (1 << i)) != 0;
                            print!("{}", if bit_value { "1" } else { "0" });
                        }
                        println!(" (预期: 10101010)");
                    }
                }
                Err(e) => {
                    println!("  ⚠️  验证读取失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 写入多个线圈失败: {}", e);
        }
    }
    Ok(())
}

async fn test_write_multiple_registers(transport: &mut TcpTransport) -> ModbusResult<()> {
    // 创建写入多个寄存器的请求
    let test_values: Vec<u16> = vec![0x1111, 0x2222, 0x3333];
    let mut data = Vec::new();
    data.push(6); // 字节数 (3个寄存器 * 2字节)
    
    for value in &test_values {
        data.extend_from_slice(&value.to_be_bytes());
    }
    
    let request = ModbusRequest {
        slave_id: 1,
        function: ModbusFunction::WriteMultipleRegisters,
        address: 400,
        quantity: 3,
        data,
    };
    
    match transport.request(&request).await {
        Ok(response) => {
            println!("  ✅ 写入多个寄存器成功");
            println!("  📝 写入地址400-402, 值: 0x1111, 0x2222, 0x3333");
            println!("  📦 响应数据长度: {} bytes", response.data.len());
            
            // 验证写入 - 读取刚写入的寄存器
            let read_request = ModbusRequest::new_read(1, ModbusFunction::ReadHoldingRegisters, 400, 3);
            match transport.request(&read_request).await {
                Ok(read_response) => {
                    if read_response.data.len() >= 7 {
                        print!("  🔍 验证读取: ");
                        for i in 0..3 {
                            let offset = 1 + i * 2;
                            let value = u16::from_be_bytes([read_response.data[offset], read_response.data[offset + 1]]);
                            print!("0x{:04X} ", value);
                        }
                        println!("(预期: 0x1111 0x2222 0x3333)");
                    }
                }
                Err(e) => {
                    println!("  ⚠️  验证读取失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 写入多个寄存器失败: {}", e);
        }
    }
    Ok(())
} 