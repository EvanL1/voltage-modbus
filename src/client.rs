/// High-level Modbus client implementations
/// 
/// This module provides user-friendly client interfaces for Modbus communication,
/// abstracting away the low-level protocol details.
/// 
/// The key insight is that Modbus TCP and RTU share the same application layer (PDU),
/// differing only in transport layer encapsulation:
/// - TCP: MBAP Header + PDU
/// - RTU: Slave ID + PDU + CRC
/// 
/// This allows us to implement the application logic once and reuse it for both transports.

use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use async_trait::async_trait;

use crate::error::{ModbusError, ModbusResult};
use crate::protocol::{ModbusRequest, ModbusResponse, ModbusFunction, SlaveId, data_utils};
use crate::transport::{ModbusTransport, TcpTransport, RtuTransport, TransportStats};

/// High-level Modbus client trait
#[async_trait]
pub trait ModbusClient: Send + Sync {
    /// Read coils (function code 0x01)
    async fn read_coils(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>>;
    
    /// Read discrete inputs (function code 0x02)
    async fn read_discrete_inputs(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>>;
    
    /// Read holding registers (function code 0x03)
    async fn read_holding_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>>;
    
    /// Read input registers (function code 0x04)
    async fn read_input_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>>;
    
    /// Write single coil (function code 0x05)
    async fn write_single_coil(&mut self, slave_id: SlaveId, address: u16, value: bool) -> ModbusResult<()>;
    
    /// Write single register (function code 0x06)
    async fn write_single_register(&mut self, slave_id: SlaveId, address: u16, value: u16) -> ModbusResult<()>;
    
    /// Write multiple coils (function code 0x0F)
    async fn write_multiple_coils(&mut self, slave_id: SlaveId, address: u16, values: &[bool]) -> ModbusResult<()>;
    
    /// Write multiple registers (function code 0x10)
    async fn write_multiple_registers(&mut self, slave_id: SlaveId, address: u16, values: &[u16]) -> ModbusResult<()>;
    
    /// Check if client is connected
    fn is_connected(&self) -> bool;
    
    /// Close the client connection
    async fn close(&mut self) -> ModbusResult<()>;
    
    /// Get transport statistics
    fn get_stats(&self) -> TransportStats;
}

/// Generic Modbus client that works with any transport
/// 
/// This client implements the common application layer logic (PDU construction and parsing)
/// while delegating transport-specific concerns to the underlying transport implementation.
/// This eliminates code duplication between TCP and RTU clients since the PDU is identical.
pub struct GenericModbusClient<T: ModbusTransport> {
    transport: T,
}

impl<T: ModbusTransport> GenericModbusClient<T> {
    /// Create a new generic client with the specified transport
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
    
    /// Get a reference to the underlying transport
    pub fn transport(&self) -> &T {
        &self.transport
    }
    
    /// Get a mutable reference to the underlying transport
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }
    
    /// Execute a raw request
    pub async fn execute_request(&mut self, request: ModbusRequest) -> ModbusResult<ModbusResponse> {
        self.transport.request(&request).await
    }
}

#[async_trait]
impl<T: ModbusTransport + Send + Sync> ModbusClient for GenericModbusClient<T> {
    async fn read_coils(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        if quantity == 0 || quantity > crate::MAX_COILS_PER_REQUEST {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        let request = ModbusRequest::new_read(
            slave_id,
            ModbusFunction::ReadCoils,
            address,
            quantity,
        );
        
        let response = self.transport.request(&request).await?;
        let bits = response.parse_bits()?;
        
        // Return only the requested number of bits
        Ok(bits.into_iter().take(quantity as usize).collect())
    }
    
    async fn read_discrete_inputs(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        if quantity == 0 || quantity > crate::MAX_COILS_PER_REQUEST {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        let request = ModbusRequest::new_read(
            slave_id,
            ModbusFunction::ReadDiscreteInputs,
            address,
            quantity,
        );
        
        let response = self.transport.request(&request).await?;
        let bits = response.parse_bits()?;
        
        // Return only the requested number of bits
        Ok(bits.into_iter().take(quantity as usize).collect())
    }
    
    async fn read_holding_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        if quantity == 0 || quantity > crate::MAX_REGISTERS_PER_REQUEST {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        let request = ModbusRequest::new_read(
            slave_id,
            ModbusFunction::ReadHoldingRegisters,
            address,
            quantity,
        );
        
        let response = self.transport.request(&request).await?;
        response.parse_registers()
    }
    
    async fn read_input_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        if quantity == 0 || quantity > crate::MAX_REGISTERS_PER_REQUEST {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        let request = ModbusRequest::new_read(
            slave_id,
            ModbusFunction::ReadInputRegisters,
            address,
            quantity,
        );
        
        let response = self.transport.request(&request).await?;
        response.parse_registers()
    }
    
    async fn write_single_coil(&mut self, slave_id: SlaveId, address: u16, value: bool) -> ModbusResult<()> {
        let data = vec![if value { 1 } else { 0 }];
        let request = ModbusRequest::new_write(
            slave_id,
            ModbusFunction::WriteSingleCoil,
            address,
            data,
        );
        
        let _response = self.transport.request(&request).await?;
        Ok(())
    }
    
    async fn write_single_register(&mut self, slave_id: SlaveId, address: u16, value: u16) -> ModbusResult<()> {
        let data = value.to_be_bytes().to_vec();
        let request = ModbusRequest::new_write(
            slave_id,
            ModbusFunction::WriteSingleRegister,
            address,
            data,
        );
        
        let _response = self.transport.request(&request).await?;
        Ok(())
    }
    
    async fn write_multiple_coils(&mut self, slave_id: SlaveId, address: u16, values: &[bool]) -> ModbusResult<()> {
        if values.is_empty() || values.len() > crate::MAX_COILS_PER_REQUEST as usize {
            return Err(ModbusError::invalid_address(address, values.len() as u16));
        }
        
        let data = data_utils::pack_bits(values);
        let request = ModbusRequest::new_write(
            slave_id,
            ModbusFunction::WriteMultipleCoils,
            address,
            data,
        );
        
        let _response = self.transport.request(&request).await?;
        Ok(())
    }
    
    async fn write_multiple_registers(&mut self, slave_id: SlaveId, address: u16, values: &[u16]) -> ModbusResult<()> {
        if values.is_empty() || values.len() > crate::MAX_REGISTERS_PER_REQUEST as usize {
            return Err(ModbusError::invalid_address(address, values.len() as u16));
        }
        
        let data = data_utils::registers_to_bytes(values);
        let request = ModbusRequest::new_write(
            slave_id,
            ModbusFunction::WriteMultipleRegisters,
            address,
            data,
        );
        
        let _response = self.transport.request(&request).await?;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.transport.is_connected()
    }
    
    async fn close(&mut self) -> ModbusResult<()> {
        self.transport.close().await
    }
    
    fn get_stats(&self) -> TransportStats {
        self.transport.get_stats()
    }
}

/// Modbus TCP client implementation using the generic client
pub struct ModbusTcpClient {
    inner: GenericModbusClient<TcpTransport>,
}

impl ModbusTcpClient {
    /// Create a new Modbus TCP client
    pub async fn new(address: &str) -> ModbusResult<Self> {
        Self::with_timeout(address, Duration::from_millis(crate::DEFAULT_TIMEOUT_MS)).await
    }
    
    /// Create a new Modbus TCP client with custom timeout
    pub async fn with_timeout(address: &str, timeout: Duration) -> ModbusResult<Self> {
        let socket_addr = SocketAddr::from_str(address)
            .map_err(|e| ModbusError::configuration(format!("Invalid address '{}': {}", address, e)))?;
            
        let transport = TcpTransport::new(socket_addr, timeout).await?;
        
        Ok(Self { inner: GenericModbusClient::new(transport) })
    }
    
    /// Create from existing TcpTransport
    pub fn from_transport(transport: TcpTransport) -> Self {
        Self { inner: GenericModbusClient::new(transport) }
    }
    
    /// Get the server address
    pub fn server_address(&self) -> SocketAddr {
        self.inner.transport().address
    }
    
    /// Execute a raw request
    pub async fn execute_request(&mut self, request: ModbusRequest) -> ModbusResult<ModbusResponse> {
        self.inner.execute_request(request).await
    }
}

#[async_trait]
impl ModbusClient for ModbusTcpClient {
    async fn read_coils(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        self.inner.read_coils(slave_id, address, quantity).await
    }
    
    async fn read_discrete_inputs(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        self.inner.read_discrete_inputs(slave_id, address, quantity).await
    }
    
    async fn read_holding_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        self.inner.read_holding_registers(slave_id, address, quantity).await
    }
    
    async fn read_input_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        self.inner.read_input_registers(slave_id, address, quantity).await
    }
    
    async fn write_single_coil(&mut self, slave_id: SlaveId, address: u16, value: bool) -> ModbusResult<()> {
        self.inner.write_single_coil(slave_id, address, value).await
    }
    
    async fn write_single_register(&mut self, slave_id: SlaveId, address: u16, value: u16) -> ModbusResult<()> {
        self.inner.write_single_register(slave_id, address, value).await
    }
    
    async fn write_multiple_coils(&mut self, slave_id: SlaveId, address: u16, values: &[bool]) -> ModbusResult<()> {
        self.inner.write_multiple_coils(slave_id, address, values).await
    }
    
    async fn write_multiple_registers(&mut self, slave_id: SlaveId, address: u16, values: &[u16]) -> ModbusResult<()> {
        self.inner.write_multiple_registers(slave_id, address, values).await
    }
    
    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
    
    async fn close(&mut self) -> ModbusResult<()> {
        self.inner.close().await
    }
    
    fn get_stats(&self) -> TransportStats {
        self.inner.get_stats()
    }
}

/// Modbus RTU client implementation using the generic client
pub struct ModbusRtuClient {
    inner: GenericModbusClient<RtuTransport>,
}

impl ModbusRtuClient {
    /// Create a new Modbus RTU client with default settings
    pub fn new(port: &str, baud_rate: u32) -> ModbusResult<Self> {
        let transport = RtuTransport::new(port, baud_rate)?;
        Ok(Self { inner: GenericModbusClient::new(transport) })
    }
    
    /// Create a new Modbus RTU client with custom configuration
    pub fn with_config(
        port: &str,
        baud_rate: u32,
        data_bits: tokio_serial::DataBits,
        stop_bits: tokio_serial::StopBits,
        parity: tokio_serial::Parity,
        timeout: Duration,
    ) -> ModbusResult<Self> {
        let transport = RtuTransport::new_with_config(
            port, baud_rate, data_bits, stop_bits, parity, timeout
        )?;
        Ok(Self { inner: GenericModbusClient::new(transport) })
    }
    
    /// Create from existing RtuTransport
    pub fn from_transport(transport: RtuTransport) -> Self {
        Self { inner: GenericModbusClient::new(transport) }
    }
    
    /// Get the transport reference
    pub fn transport(&self) -> &RtuTransport {
        self.inner.transport()
    }
    
    /// Execute a raw request
    pub async fn execute_request(&mut self, request: ModbusRequest) -> ModbusResult<ModbusResponse> {
        self.inner.execute_request(request).await
    }
}

#[async_trait]
impl ModbusClient for ModbusRtuClient {
    async fn read_coils(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        self.inner.read_coils(slave_id, address, quantity).await
    }
    
    async fn read_discrete_inputs(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        self.inner.read_discrete_inputs(slave_id, address, quantity).await
    }
    
    async fn read_holding_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        self.inner.read_holding_registers(slave_id, address, quantity).await
    }
    
    async fn read_input_registers(&mut self, slave_id: SlaveId, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        self.inner.read_input_registers(slave_id, address, quantity).await
    }
    
    async fn write_single_coil(&mut self, slave_id: SlaveId, address: u16, value: bool) -> ModbusResult<()> {
        self.inner.write_single_coil(slave_id, address, value).await
    }
    
    async fn write_single_register(&mut self, slave_id: SlaveId, address: u16, value: u16) -> ModbusResult<()> {
        self.inner.write_single_register(slave_id, address, value).await
    }
    
    async fn write_multiple_coils(&mut self, slave_id: SlaveId, address: u16, values: &[bool]) -> ModbusResult<()> {
        self.inner.write_multiple_coils(slave_id, address, values).await
    }
    
    async fn write_multiple_registers(&mut self, slave_id: SlaveId, address: u16, values: &[u16]) -> ModbusResult<()> {
        self.inner.write_multiple_registers(slave_id, address, values).await
    }
    
    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
    
    async fn close(&mut self) -> ModbusResult<()> {
        self.inner.close().await
    }
    
    fn get_stats(&self) -> TransportStats {
        self.inner.get_stats()
    }
}

/// High-level utility functions for common operations
pub mod utils {
    use super::*;
    
    /// Read multiple register types in a single operation
    pub async fn read_mixed_registers<T: ModbusClient>(
        client: &mut T,
        slave_id: SlaveId,
        operations: &[(ModbusFunction, u16, u16)], // (function, address, quantity)
    ) -> ModbusResult<Vec<Vec<u16>>> {
        let mut results = Vec::new();
        
        for &(function, address, quantity) in operations {
            let values = match function {
                ModbusFunction::ReadHoldingRegisters => {
                    client.read_holding_registers(slave_id, address, quantity).await?
                },
                ModbusFunction::ReadInputRegisters => {
                    client.read_input_registers(slave_id, address, quantity).await?
                },
                _ => return Err(ModbusError::invalid_function(function.to_u8())),
            };
            results.push(values);
        }
        
        Ok(results)
    }
    
    /// Batch write multiple registers
    pub async fn batch_write_registers<T: ModbusClient>(
        client: &mut T,
        slave_id: SlaveId,
        writes: &[(u16, Vec<u16>)], // (address, values)
    ) -> ModbusResult<()> {
        for (address, values) in writes {
            if values.len() == 1 {
                client.write_single_register(slave_id, *address, values[0]).await?;
            } else {
                client.write_multiple_registers(slave_id, *address, values).await?;
            }
        }
        Ok(())
    }
    
    /// Convert register values to different data types
    pub fn registers_to_u32_be(registers: &[u16]) -> Vec<u32> {
        registers.chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(((chunk[0] as u32) << 16) | (chunk[1] as u32))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Convert register values to i32 (big-endian)
    pub fn registers_to_i32_be(registers: &[u16]) -> Vec<i32> {
        registers_to_u32_be(registers)
            .into_iter()
            .map(|v| v as i32)
            .collect()
    }
    
    /// Convert register values to f32 (IEEE 754, big-endian)
    pub fn registers_to_f32_be(registers: &[u16]) -> Vec<f32> {
        registers_to_u32_be(registers)
            .into_iter()
            .map(|v| f32::from_bits(v))
            .collect()
    }
    
    /// Convert u32 values to register pairs (big-endian)
    pub fn u32_to_registers_be(values: &[u32]) -> Vec<u16> {
        values.iter()
            .flat_map(|&v| [(v >> 16) as u16, v as u16])
            .collect()
    }
    
    /// Convert f32 values to register pairs (IEEE 754, big-endian)
    pub fn f32_to_registers_be(values: &[f32]) -> Vec<u16> {
        let u32_values: Vec<u32> = values.iter().map(|&v| v.to_bits()).collect();
        u32_to_registers_be(&u32_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_conversion() {
        let registers = vec![0x1234, 0x5678, 0xABCD, 0xEF01];
        let u32_values = utils::registers_to_u32_be(&registers);
        assert_eq!(u32_values, vec![0x12345678, 0xABCDEF01]);
        
        let back_to_registers = utils::u32_to_registers_be(&u32_values);
        assert_eq!(back_to_registers, registers);
    }
    
    #[test]
    fn test_float_conversion() {
        let float_values = vec![1.5f32, -2.75f32];
        let registers = utils::f32_to_registers_be(&float_values);
        let back_to_floats = utils::registers_to_f32_be(&registers);
        
        for (original, converted) in float_values.iter().zip(back_to_floats.iter()) {
            assert!((original - converted).abs() < f32::EPSILON);
        }
    }
    
    #[tokio::test]
    async fn test_tcp_client_creation() {
        // Test with invalid address
        let result = ModbusTcpClient::new("invalid_address").await;
        assert!(result.is_err());
        
        // Test with valid but non-existent address
        let result = ModbusTcpClient::new("127.0.0.1:9999").await;
        // This might fail due to connection refused, which is expected
        println!("TCP client creation result: {:?}", result.is_ok());
    }
    
    #[test]
    fn test_rtu_client_creation() {
        // Test RTU client creation (will fail if no serial port available)
        let result = ModbusRtuClient::new("/dev/ttyUSB0", 9600);
        println!("RTU client creation result: {:?}", result.is_ok());
        
        // Test with custom configuration
        let result = ModbusRtuClient::with_config(
            "/dev/ttyUSB0",
            9600,
            tokio_serial::DataBits::Eight,
            tokio_serial::StopBits::One,
            tokio_serial::Parity::None,
            Duration::from_secs(1),
        );
        println!("RTU client with config creation result: {:?}", result.is_ok());
    }
    
    #[tokio::test]
    async fn test_rtu_client_operations() {
        // This test will only pass if a serial port is available
        // In a real environment, you would have a Modbus RTU device connected
        
        // Try to create RTU client - this might fail if no port is available
        let client_result = ModbusRtuClient::new("/dev/ttyUSB0", 9600);
        
        if let Ok(mut client) = client_result {
            // Test connection status
            println!("RTU client connected: {}", client.is_connected());
            
            // Test reading coils (this will likely timeout without a real device)
            let read_result = tokio::time::timeout(
                Duration::from_millis(100),
                client.read_coils(1, 0, 8)
            ).await;
            
            match read_result {
                Ok(Ok(coils)) => {
                    println!("Successfully read {} coils", coils.len());
                }
                Ok(Err(e)) => {
                    println!("Read operation failed (expected without device): {}", e);
                }
                Err(_) => {
                    println!("Read operation timed out (expected without device)");
                }
            }
            
            // Close the client
            let _ = client.close().await;
        } else {
            println!("RTU client creation failed (expected without serial port)");
        }
    }
    
    #[test]
    fn test_rtu_client_configuration() {
        // Test different configurations
        let configs = vec![
            ("/dev/ttyUSB0", 9600),
            ("/dev/ttyUSB1", 19200),
            ("/dev/ttyS0", 38400),
            ("COM1", 115200),
        ];
        
        for (port, baud) in configs {
            let result = ModbusRtuClient::new(port, baud);
            // We expect these to fail without actual hardware, but they should not panic
            println!("RTU client creation for {} at {} baud: {}", port, baud, result.is_ok());
        }
    }
} 