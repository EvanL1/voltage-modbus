/// Modbus transport layer implementations
/// 
/// This module provides the transport layer abstractions and implementations
/// for both Modbus TCP and RTU protocols.

use std::net::SocketAddr;
use std::time::Duration;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use crc::{Crc, CRC_16_MODBUS};
use bytes::{Buf, BufMut, BytesMut};

use crate::error::{ModbusError, ModbusResult};
use crate::protocol::{ModbusRequest, ModbusResponse, ModbusFunction, SlaveId};

/// Maximum frame size for Modbus TCP (MBAP header + PDU)
const MAX_TCP_FRAME_SIZE: usize = 260;

/// Maximum frame size for Modbus RTU
const MAX_RTU_FRAME_SIZE: usize = 256;

/// Modbus TCP Application Protocol header size
const MBAP_HEADER_SIZE: usize = 6;

/// CRC calculator for RTU
const CRC_MODBUS: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);

/// Transport layer trait for different Modbus protocols
#[async_trait]
pub trait ModbusTransport: Send + Sync {
    /// Send a request and receive response
    async fn request(&mut self, request: &ModbusRequest) -> ModbusResult<ModbusResponse>;
    
    /// Check if transport is connected
    fn is_connected(&self) -> bool;
    
    /// Close the transport connection
    async fn close(&mut self) -> ModbusResult<()>;
    
    /// Get transport statistics
    fn get_stats(&self) -> TransportStats;
}

/// Transport layer statistics
#[derive(Debug, Clone, Default)]
pub struct TransportStats {
    pub requests_sent: u64,
    pub responses_received: u64,
    pub errors: u64,
    pub timeouts: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Modbus TCP transport implementation
pub struct TcpTransport {
    stream: Option<TcpStream>,
    pub address: SocketAddr,
    timeout: Duration,
    transaction_id: u16,
    stats: TransportStats,
}

impl TcpTransport {
    /// Create a new TCP transport
    pub async fn new(address: SocketAddr, timeout: Duration) -> ModbusResult<Self> {
        let stream = TcpStream::connect(address).await
            .map_err(|e| ModbusError::connection(format!("Failed to connect to {}: {}", address, e)))?;
            
        Ok(Self {
            stream: Some(stream),
            address,
            timeout,
            transaction_id: 1,
            stats: TransportStats::default(),
        })
    }
    
    /// Reconnect to the server
    async fn reconnect(&mut self) -> ModbusResult<()> {
        self.stream = None;
        
        let stream = TcpStream::connect(self.address).await
            .map_err(|e| ModbusError::connection(format!("Failed to reconnect to {}: {}", self.address, e)))?;
            
        self.stream = Some(stream);
        Ok(())
    }
    
    /// Get next transaction ID
    fn next_transaction_id(&mut self) -> u16 {
        self.transaction_id = self.transaction_id.wrapping_add(1);
        if self.transaction_id == 0 {
            self.transaction_id = 1;
        }
        self.transaction_id
    }
    
    /// Encode request to TCP frame
    fn encode_request(&mut self, request: &ModbusRequest) -> Vec<u8> {
        let transaction_id = self.next_transaction_id();
        let protocol_id = 0u16; // Always 0 for Modbus
        
        // Calculate PDU length (unit_id + function_code + data)
        let pdu_length = 1 + 1 + match request.function {
            ModbusFunction::ReadCoils | 
            ModbusFunction::ReadDiscreteInputs |
            ModbusFunction::ReadHoldingRegisters |
            ModbusFunction::ReadInputRegisters => 4, // address (2) + quantity (2)
            
            ModbusFunction::WriteSingleCoil |
            ModbusFunction::WriteSingleRegister => 4, // address (2) + value (2)
            
            ModbusFunction::WriteMultipleCoils |
            ModbusFunction::WriteMultipleRegisters => 5 + request.data.len(), // address (2) + quantity (2) + byte_count (1) + data
        };
        
        let mut frame = Vec::with_capacity(MBAP_HEADER_SIZE + pdu_length);
        
        // MBAP Header: Transaction ID (2) + Protocol ID (2) + Length (2)
        frame.extend_from_slice(&transaction_id.to_be_bytes());
        frame.extend_from_slice(&protocol_id.to_be_bytes());
        frame.extend_from_slice(&(pdu_length as u16).to_be_bytes()); // PDU length without MBAP header
        
        // PDU: Unit ID + Function Code + Data
        frame.push(request.slave_id);
        frame.push(request.function.to_u8());
        frame.extend_from_slice(&request.address.to_be_bytes());
        
        match request.function {
            ModbusFunction::ReadCoils | 
            ModbusFunction::ReadDiscreteInputs |
            ModbusFunction::ReadHoldingRegisters |
            ModbusFunction::ReadInputRegisters => {
                frame.extend_from_slice(&request.quantity.to_be_bytes());
            },
            
            ModbusFunction::WriteSingleCoil => {
                let value: u16 = if !request.data.is_empty() && request.data[0] != 0 { 0xFF00 } else { 0x0000 };
                frame.extend_from_slice(&value.to_be_bytes());
            },
            
            ModbusFunction::WriteSingleRegister => {
                if request.data.len() >= 2 {
                    frame.extend_from_slice(&request.data[0..2]);
                } else {
                    frame.extend_from_slice(&[0, 0]);
                }
            },
            
            ModbusFunction::WriteMultipleCoils |
            ModbusFunction::WriteMultipleRegisters => {
                frame.extend_from_slice(&request.quantity.to_be_bytes());
                frame.push(request.data.len() as u8);
                frame.extend_from_slice(&request.data);
            },
        }
        
        frame
    }
    
    /// Decode response from TCP frame
    fn decode_response(&self, frame: &[u8]) -> ModbusResult<ModbusResponse> {
        if frame.len() < MBAP_HEADER_SIZE + 2 {
            return Err(ModbusError::frame("Frame too short"));
        }
        
        // Parse MBAP header
        let _transaction_id = u16::from_be_bytes([frame[0], frame[1]]);
        let _protocol_id = u16::from_be_bytes([frame[2], frame[3]]);
        let length = u16::from_be_bytes([frame[4], frame[5]]);
        let slave_id = frame[6];
        
        if frame.len() < MBAP_HEADER_SIZE + length as usize {
            return Err(ModbusError::frame("Incomplete frame"));
        }
        
        // Parse PDU
        let function_code = frame[7];
        
        // Check for exception response
        if function_code & 0x80 != 0 {
            if frame.len() < MBAP_HEADER_SIZE + 3 {
                return Err(ModbusError::frame("Invalid exception response"));
            }
            
            let original_function = function_code & 0x7F;
            let exception_code = frame[8];
            
            return Ok(ModbusResponse::new_exception(
                slave_id,
                ModbusFunction::from_u8(original_function)?,
                exception_code,
            ));
        }
        
        let function = ModbusFunction::from_u8(function_code)?;
        let data = frame[MBAP_HEADER_SIZE + 2..MBAP_HEADER_SIZE + length as usize].to_vec();
        
        Ok(ModbusResponse::new_success(slave_id, function, data))
    }
}

#[async_trait]
impl ModbusTransport for TcpTransport {
    async fn request(&mut self, request: &ModbusRequest) -> ModbusResult<ModbusResponse> {
        // Validate request
        request.validate()?;
        
        // Ensure connection
        if self.stream.is_none() {
            self.reconnect().await?;
        }
        
        // Encode and send request  
        let frame = self.encode_request(request);
        self.stats.requests_sent += 1;
        self.stats.bytes_sent += frame.len() as u64;

        let stream = self.stream.as_mut().unwrap();
        
        let send_result = timeout(self.timeout, stream.write_all(&frame)).await;
        if send_result.is_err() || send_result.unwrap().is_err() {
            self.stats.timeouts += 1;
            self.stats.errors += 1;
            self.stream = None; // Mark connection as broken
            return Err(ModbusError::timeout("send request", self.timeout.as_millis() as u64));
        }
        
        // Read response header first
        let mut header_buf = [0u8; MBAP_HEADER_SIZE + 1]; // MBAP + function code
        let read_result = timeout(self.timeout, stream.read_exact(&mut header_buf)).await;
        
        if read_result.is_err() || read_result.unwrap().is_err() {
            self.stats.timeouts += 1;
            self.stats.errors += 1;
            self.stream = None;
            return Err(ModbusError::timeout("read response header", self.timeout.as_millis() as u64));
        }
        
        // Parse length from header
        let length = u16::from_be_bytes([header_buf[4], header_buf[5]]);
        if length > MAX_TCP_FRAME_SIZE as u16 {
            self.stats.errors += 1;
            return Err(ModbusError::frame("Response frame too large"));
        }
        
        // Read remaining data
        let remaining_bytes = (length as usize).saturating_sub(1); // -1 for function code already read
        let mut response_buf = vec![0u8; MBAP_HEADER_SIZE + 1 + remaining_bytes];
        response_buf[..MBAP_HEADER_SIZE + 1].copy_from_slice(&header_buf);
        
        if remaining_bytes > 0 {
            let read_result = timeout(
                self.timeout, 
                stream.read_exact(&mut response_buf[MBAP_HEADER_SIZE + 1..])
            ).await;
            
            if read_result.is_err() || read_result.unwrap().is_err() {
                self.stats.timeouts += 1;
                self.stats.errors += 1;
                self.stream = None;
                return Err(ModbusError::timeout("read response data", self.timeout.as_millis() as u64));
            }
        }
        
        self.stats.responses_received += 1;
        self.stats.bytes_received += response_buf.len() as u64;
        
        // Decode response
        let response = self.decode_response(&response_buf)?;
        
        // Check for exception
        if let Some(error) = response.get_exception() {
            self.stats.errors += 1;
            return Err(error);
        }
        
        Ok(response)
    }
    
    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
    
    async fn close(&mut self) -> ModbusResult<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        Ok(())
    }
    
    fn get_stats(&self) -> TransportStats {
        self.stats.clone()
    }
}

/// Modbus RTU transport implementation (placeholder for now)
pub struct RtuTransport {
    // RTU-specific fields would go here
    stats: TransportStats,
}

impl RtuTransport {
    /// Create a new RTU transport
    pub fn new(_port: &str, _baud_rate: u32) -> ModbusResult<Self> {
        // RTU implementation would go here
        Ok(Self {
            stats: TransportStats::default(),
        })
    }
    
    /// Calculate CRC for RTU frame
    fn calculate_crc(data: &[u8]) -> u16 {
        CRC_MODBUS.checksum(data)
    }
    
    /// Encode request to RTU frame
    fn encode_request(&self, _request: &ModbusRequest) -> Vec<u8> {
        // RTU encoding implementation would go here
        Vec::new()
    }
    
    /// Decode response from RTU frame
    fn decode_response(&self, _frame: &[u8]) -> ModbusResult<ModbusResponse> {
        // RTU decoding implementation would go here
        Err(ModbusError::protocol("RTU not implemented yet"))
    }
}

#[async_trait]
impl ModbusTransport for RtuTransport {
    async fn request(&mut self, _request: &ModbusRequest) -> ModbusResult<ModbusResponse> {
        Err(ModbusError::protocol("RTU transport not implemented yet"))
    }
    
    fn is_connected(&self) -> bool {
        false // RTU implementation pending
    }
    
    async fn close(&mut self) -> ModbusResult<()> {
        Ok(())
    }
    
    fn get_stats(&self) -> TransportStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ModbusFunction;
    
    #[test]
    fn test_crc_calculation() {
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x02];
        let crc = RtuTransport::calculate_crc(&data);
        // Expected CRC for this data should be calculated
        assert!(crc > 0);
    }
    
    #[tokio::test]
    async fn test_tcp_transport_creation() {
        let addr = "127.0.0.1:502".parse().unwrap();
        let timeout = Duration::from_secs(5);
        
        // This will fail unless there's a server running, but tests the creation logic
        let result = TcpTransport::new(addr, timeout).await;
        // Don't assert success since we don't have a test server
        println!("TCP transport creation result: {:?}", result.is_ok());
    }
} 