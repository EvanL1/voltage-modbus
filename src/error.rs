/// Modbus-specific error types
/// 
/// This module provides comprehensive error handling for all Modbus operations,
/// including network, protocol, and data validation errors.

use std::fmt;
use thiserror::Error;

/// Result type alias for Modbus operations
pub type ModbusResult<T> = Result<T, ModbusError>;

/// Comprehensive Modbus error types
#[derive(Error, Debug, Clone)]
pub enum ModbusError {
    /// I/O related errors (network, serial)
    #[error("I/O error: {message}")]
    Io { message: String },
    
    /// Connection errors
    #[error("Connection error: {message}")]
    Connection { message: String },
    
    /// Timeout errors
    #[error("Timeout after {timeout_ms}ms: {operation}")]
    Timeout { operation: String, timeout_ms: u64 },
    
    /// Protocol-level errors
    #[error("Protocol error: {message}")]
    Protocol { message: String },
    
    /// Invalid function code
    #[error("Invalid function code: {code}")]
    InvalidFunction { code: u8 },
    
    /// Invalid address range
    #[error("Invalid address: start={start}, count={count}")]
    InvalidAddress { start: u16, count: u16 },
    
    /// Invalid data value
    #[error("Invalid data: {message}")]
    InvalidData { message: String },
    
    /// CRC validation failure
    #[error("CRC validation failed: expected={expected:04X}, actual={actual:04X}")]
    CrcMismatch { expected: u16, actual: u16 },
    
    /// Modbus exception response
    #[error("Modbus exception: function={function:02X}, code={code:02X} ({message})")]
    Exception { function: u8, code: u8, message: String },
    
    /// Frame parsing errors
    #[error("Frame error: {message}")]
    Frame { message: String },
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    /// Device not responding
    #[error("Device {slave_id} not responding")]
    DeviceNotResponding { slave_id: u8 },
    
    /// Internal errors (should not occur in normal operation)
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl ModbusError {
    /// Create a new I/O error
    pub fn io<S: Into<String>>(message: S) -> Self {
        Self::Io { message: message.into() }
    }
    
    /// Create a new connection error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection { message: message.into() }
    }
    
    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(operation: S, timeout_ms: u64) -> Self {
        Self::Timeout { 
            operation: operation.into(), 
            timeout_ms 
        }
    }
    
    /// Create a new protocol error
    pub fn protocol<S: Into<String>>(message: S) -> Self {
        Self::Protocol { message: message.into() }
    }
    
    /// Create an invalid function error
    pub fn invalid_function(code: u8) -> Self {
        Self::InvalidFunction { code }
    }
    
    /// Create an invalid address error
    pub fn invalid_address(start: u16, count: u16) -> Self {
        Self::InvalidAddress { start, count }
    }
    
    /// Create an invalid data error
    pub fn invalid_data<S: Into<String>>(message: S) -> Self {
        Self::InvalidData { message: message.into() }
    }
    
    /// Create a CRC mismatch error
    pub fn crc_mismatch(expected: u16, actual: u16) -> Self {
        Self::CrcMismatch { expected, actual }
    }
    
    /// Create a Modbus exception error
    pub fn exception(function: u8, code: u8) -> Self {
        let message = match code {
            0x01 => "Illegal Function",
            0x02 => "Illegal Data Address", 
            0x03 => "Illegal Data Value",
            0x04 => "Slave Device Failure",
            0x05 => "Acknowledge",
            0x06 => "Slave Device Busy",
            0x08 => "Memory Parity Error",
            0x0A => "Gateway Path Unavailable",
            0x0B => "Gateway Target Device Failed to Respond",
            _ => "Unknown Exception",
        }.to_string();
        
        Self::Exception { function, code, message }
    }
    
    /// Create a frame error
    pub fn frame<S: Into<String>>(message: S) -> Self {
        Self::Frame { message: message.into() }
    }
    
    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration { message: message.into() }
    }
    
    /// Create a device not responding error
    pub fn device_not_responding(slave_id: u8) -> Self {
        Self::DeviceNotResponding { slave_id }
    }
    
    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal { message: message.into() }
    }
    
    /// Check if the error is recoverable (can retry)
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Io { .. } => true,
            Self::Connection { .. } => true,
            Self::Timeout { .. } => true,
            Self::DeviceNotResponding { .. } => true,
            Self::Exception { code, .. } => {
                // Some exceptions are recoverable
                matches!(code, 0x05 | 0x06) // Acknowledge, Busy
            },
            _ => false,
        }
    }
    
    /// Check if the error is a network/transport issue
    pub fn is_transport_error(&self) -> bool {
        matches!(self, 
            Self::Io { .. } | 
            Self::Connection { .. } | 
            Self::Timeout { .. }
        )
    }
    
    /// Check if the error is a protocol issue
    pub fn is_protocol_error(&self) -> bool {
        matches!(self,
            Self::Protocol { .. } |
            Self::InvalidFunction { .. } |
            Self::Exception { .. } |
            Self::Frame { .. } |
            Self::CrcMismatch { .. }
        )
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for ModbusError {
    fn from(err: std::io::Error) -> Self {
        Self::io(err.to_string())
    }
}

/// Convert from tokio timeout errors
impl From<tokio::time::error::Elapsed> for ModbusError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::timeout("Operation timeout", 0)
    }
}

/// Convert from serde JSON errors
impl From<serde_json::Error> for ModbusError {
    fn from(err: serde_json::Error) -> Self {
        Self::invalid_data(format!("JSON error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = ModbusError::timeout("read_registers", 5000);
        assert!(err.is_recoverable());
        assert!(err.is_transport_error());
        
        let err = ModbusError::exception(0x03, 0x02);
        assert!(!err.is_recoverable());
        assert!(err.is_protocol_error());
    }
    
    #[test]
    fn test_error_display() {
        let err = ModbusError::crc_mismatch(0x1234, 0x5678);
        let msg = format!("{}", err);
        assert!(msg.contains("CRC validation failed"));
        assert!(msg.contains("1234"));
        assert!(msg.contains("5678"));
    }
} 