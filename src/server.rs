/// Modbus server implementations
/// 
/// This module provides complete server-side implementations for both TCP and RTU protocols.

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, Mutex};
use tokio::time::timeout;
use log::{info, error, debug, warn};

use crate::error::{ModbusError, ModbusResult};
use crate::protocol::{ModbusRequest, ModbusResponse, ModbusFunction};
use crate::register_bank::{ModbusRegisterBank, RegisterBankStats};

/// Maximum frame size for Modbus TCP
const MAX_TCP_FRAME_SIZE: usize = 260;

/// MBAP header size
const MBAP_HEADER_SIZE: usize = 6;

/// Modbus server trait
#[async_trait]
pub trait ModbusServer: Send + Sync {
    /// Start the server
    async fn start(&mut self) -> ModbusResult<()>;
    
    /// Stop the server
    async fn stop(&mut self) -> ModbusResult<()>;
    
    /// Check if server is running
    fn is_running(&self) -> bool;
    
    /// Get server statistics
    fn get_stats(&self) -> ServerStats;
    
    /// Get register bank reference
    fn get_register_bank(&self) -> Option<Arc<ModbusRegisterBank>>;
}

/// Server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub connections_count: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub uptime_seconds: u64,
    pub register_bank_stats: Option<RegisterBankStats>,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self {
            connections_count: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            bytes_received: 0,
            bytes_sent: 0,
            uptime_seconds: 0,
            register_bank_stats: None,
        }
    }
}

/// Modbus TCP server configuration
#[derive(Debug, Clone)]
pub struct ModbusTcpServerConfig {
    pub bind_address: SocketAddr,
    pub max_connections: usize,
    pub request_timeout: Duration,
    pub register_bank: Option<Arc<ModbusRegisterBank>>,
}

impl Default for ModbusTcpServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:502".parse().unwrap(),
            max_connections: 100,
            request_timeout: Duration::from_secs(30),
            register_bank: None,
        }
    }
}

/// Modbus TCP server implementation
pub struct ModbusTcpServer {
    config: ModbusTcpServerConfig,
    register_bank: Arc<ModbusRegisterBank>,
    stats: Arc<Mutex<ServerStats>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
    is_running: Arc<Mutex<bool>>,
    start_time: Option<std::time::Instant>,
}

impl ModbusTcpServer {
    /// Create a new TCP server with default configuration
    pub fn new(bind_address: &str) -> ModbusResult<Self> {
        let addr = bind_address.parse()
            .map_err(|e| ModbusError::invalid_data(format!("Invalid bind address: {}", e)))?;
        
        let config = ModbusTcpServerConfig {
            bind_address: addr,
            ..Default::default()
        };
        
        Self::with_config(config)
    }
    
    /// Create a new TCP server with custom configuration
    pub fn with_config(config: ModbusTcpServerConfig) -> ModbusResult<Self> {
        let register_bank = config.register_bank.clone()
            .unwrap_or_else(|| Arc::new(ModbusRegisterBank::new()));
        
        Ok(Self {
            config,
            register_bank,
            stats: Arc::new(Mutex::new(ServerStats::default())),
            shutdown_tx: None,
            is_running: Arc::new(Mutex::new(false)),
            start_time: None,
        })
    }
    
    /// Set custom register bank
    pub fn set_register_bank(&mut self, register_bank: Arc<ModbusRegisterBank>) {
        self.register_bank = register_bank;
    }
    
    /// Handle client connection
    async fn handle_client(
        stream: TcpStream,
        register_bank: Arc<ModbusRegisterBank>,
        stats: Arc<Mutex<ServerStats>>,
        mut shutdown_rx: broadcast::Receiver<()>,
        request_timeout: Duration,
    ) {
        let peer_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
        info!("📡 New client connected: {}", peer_addr);
        
        // Update connection count
        {
            let mut stats = stats.lock().await;
            stats.connections_count += 1;
        }
        
        let mut stream = stream;
        let mut buffer = vec![0u8; MAX_TCP_FRAME_SIZE];
        
        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = shutdown_rx.recv() => {
                    debug!("Shutdown signal received for client {}", peer_addr);
                    break;
                }
                
                // Handle client request
                result = timeout(request_timeout, stream.read(&mut buffer)) => {
                    match result {
                        Ok(Ok(0)) => {
                            debug!("Client {} disconnected", peer_addr);
                            break;
                        }
                        Ok(Ok(bytes_read)) => {
                            // Update stats
                            {
                                let mut stats = stats.lock().await;
                                stats.total_requests += 1;
                                stats.bytes_received += bytes_read as u64;
                            }
                            
                            // Process request
                            match Self::process_request(&buffer[..bytes_read], &register_bank).await {
                                Ok(response_data) => {
                                    if let Err(e) = stream.write_all(&response_data).await {
                                        error!("Failed to send response to {}: {}", peer_addr, e);
                                        break;
                                    } else {
                                        // Update success stats
                                        let mut stats = stats.lock().await;
                                        stats.successful_requests += 1;
                                        stats.bytes_sent += response_data.len() as u64;
                                    }
                                }
                                Err(e) => {
                                    error!("Error processing request from {}: {}", peer_addr, e);
                                    
                                    // Send error response if possible
                                    if let Ok(error_response) = Self::create_error_response(&buffer[..bytes_read], 0x01) {
                                        let _ = stream.write_all(&error_response).await;
                                    }
                                    
                                    // Update error stats
                                    let mut stats = stats.lock().await;
                                    stats.failed_requests += 1;
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Read error from {}: {}", peer_addr, e);
                            break;
                        }
                        Err(_) => {
                            warn!("Read timeout from {}", peer_addr);
                            break;
                        }
                    }
                }
            }
        }
        
        info!("🔌 Client {} disconnected", peer_addr);
    }
    
    /// Process Modbus request
    async fn process_request(
        frame: &[u8],
        register_bank: &Arc<ModbusRegisterBank>,
    ) -> ModbusResult<Vec<u8>> {
        if frame.len() < MBAP_HEADER_SIZE + 2 {
            return Err(ModbusError::frame("Frame too short"));
        }
        
        // Parse MBAP header
        let transaction_id = u16::from_be_bytes([frame[0], frame[1]]);
        let protocol_id = u16::from_be_bytes([frame[2], frame[3]]);
        let length = u16::from_be_bytes([frame[4], frame[5]]);
        let unit_id = frame[6];
        let function_code = frame[7];
        
        if protocol_id != 0 {
            return Err(ModbusError::frame("Invalid protocol ID"));
        }
        
        if frame.len() < MBAP_HEADER_SIZE + length as usize {
            return Err(ModbusError::frame("Incomplete frame"));
        }
        
        debug!("Processing request: TID={}, Function=0x{:02x}, Unit={}", 
               transaction_id, function_code, unit_id);
        
        // Parse function and data
        let data = &frame[MBAP_HEADER_SIZE + 2..MBAP_HEADER_SIZE + length as usize];
        
        // Process based on function code
        let response_data = match function_code {
            0x01 => Self::handle_read_coils(data, register_bank).await?,
            0x02 => Self::handle_read_discrete_inputs(data, register_bank).await?,
            0x03 => Self::handle_read_holding_registers(data, register_bank).await?,
            0x04 => Self::handle_read_input_registers(data, register_bank).await?,
            0x05 => Self::handle_write_single_coil(data, register_bank).await?,
            0x06 => Self::handle_write_single_register(data, register_bank).await?,
            0x0F => Self::handle_write_multiple_coils(data, register_bank).await?,
            0x10 => Self::handle_write_multiple_registers(data, register_bank).await?,
            _ => {
                return Err(ModbusError::protocol(format!("Unsupported function code: 0x{:02x}", function_code)));
            }
        };
        
        // Create response frame
        let response_length = response_data.len() + 2; // +2 for unit_id and function_code
        let mut response = Vec::with_capacity(MBAP_HEADER_SIZE + response_length);
        
        // MBAP header
        response.extend_from_slice(&transaction_id.to_be_bytes());
        response.extend_from_slice(&protocol_id.to_be_bytes());
        response.extend_from_slice(&(response_length as u16).to_be_bytes());
        
        // PDU
        response.push(unit_id);
        response.push(function_code);
        response.extend_from_slice(&response_data);
        
        Ok(response)
    }
    
    /// Handle read coils (0x01)
    async fn handle_read_coils(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(ModbusError::frame("Invalid read coils request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let quantity = u16::from_be_bytes([data[2], data[3]]);
        
        if quantity == 0 || quantity > 2000 {
            return Err(ModbusError::invalid_data("Invalid quantity"));
        }
        
        let coils = register_bank.read_coils(address, quantity)?;
        
        // Pack bits into bytes
        let byte_count = (quantity + 7) / 8;
        let mut response = vec![byte_count as u8];
        
        for i in 0..byte_count {
            let mut byte_value = 0u8;
            for bit in 0..8 {
                let coil_index = (i * 8 + bit) as usize;
                if coil_index < coils.len() && coils[coil_index] {
                    byte_value |= 1 << bit;
                }
            }
            response.push(byte_value);
        }
        
        Ok(response)
    }
    
    /// Handle read discrete inputs (0x02)
    async fn handle_read_discrete_inputs(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(ModbusError::frame("Invalid read discrete inputs request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let quantity = u16::from_be_bytes([data[2], data[3]]);
        
        if quantity == 0 || quantity > 2000 {
            return Err(ModbusError::invalid_data("Invalid quantity"));
        }
        
        let inputs = register_bank.read_discrete_inputs(address, quantity)?;
        
        // Pack bits into bytes
        let byte_count = (quantity + 7) / 8;
        let mut response = vec![byte_count as u8];
        
        for i in 0..byte_count {
            let mut byte_value = 0u8;
            for bit in 0..8 {
                let input_index = (i * 8 + bit) as usize;
                if input_index < inputs.len() && inputs[input_index] {
                    byte_value |= 1 << bit;
                }
            }
            response.push(byte_value);
        }
        
        Ok(response)
    }
    
    /// Handle read holding registers (0x03)
    async fn handle_read_holding_registers(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(ModbusError::frame("Invalid read holding registers request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let quantity = u16::from_be_bytes([data[2], data[3]]);
        
        if quantity == 0 || quantity > 125 {
            return Err(ModbusError::invalid_data("Invalid quantity"));
        }
        
        let registers = register_bank.read_holding_registers(address, quantity)?;
        
        let byte_count = quantity * 2;
        let mut response = vec![byte_count as u8];
        
        for register in registers {
            response.extend_from_slice(&register.to_be_bytes());
        }
        
        Ok(response)
    }
    
    /// Handle read input registers (0x04)
    async fn handle_read_input_registers(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(ModbusError::frame("Invalid read input registers request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let quantity = u16::from_be_bytes([data[2], data[3]]);
        
        if quantity == 0 || quantity > 125 {
            return Err(ModbusError::invalid_data("Invalid quantity"));
        }
        
        let registers = register_bank.read_input_registers(address, quantity)?;
        
        let byte_count = quantity * 2;
        let mut response = vec![byte_count as u8];
        
        for register in registers {
            response.extend_from_slice(&register.to_be_bytes());
        }
        
        Ok(response)
    }
    
    /// Handle write single coil (0x05)
    async fn handle_write_single_coil(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(ModbusError::frame("Invalid write single coil request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let value = u16::from_be_bytes([data[2], data[3]]);
        
        let coil_value = match value {
            0x0000 => false,
            0xFF00 => true,
            _ => return Err(ModbusError::invalid_data("Invalid coil value")),
        };
        
        register_bank.write_single_coil(address, coil_value)?;
        
        // Echo back the request
        Ok(data.to_vec())
    }
    
    /// Handle write single register (0x06)
    async fn handle_write_single_register(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(ModbusError::frame("Invalid write single register request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let value = u16::from_be_bytes([data[2], data[3]]);
        
        register_bank.write_single_register(address, value)?;
        
        // Echo back the request
        Ok(data.to_vec())
    }
    
    /// Handle write multiple coils (0x0F)
    async fn handle_write_multiple_coils(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 5 {
            return Err(ModbusError::frame("Invalid write multiple coils request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let quantity = u16::from_be_bytes([data[2], data[3]]);
        let byte_count = data[4] as usize;
        
        if data.len() < 5 + byte_count {
            return Err(ModbusError::frame("Incomplete write multiple coils request"));
        }
        
        // Extract coil values from bytes
        let mut coils = Vec::with_capacity(quantity as usize);
        for i in 0..quantity {
            let byte_index = (i / 8) as usize;
            let bit_index = i % 8;
            let byte_value = data[5 + byte_index];
            let coil_value = (byte_value & (1 << bit_index)) != 0;
            coils.push(coil_value);
        }
        
        register_bank.write_multiple_coils(address, &coils)?;
        
        // Return address and quantity
        Ok(data[0..4].to_vec())
    }
    
    /// Handle write multiple registers (0x10)
    async fn handle_write_multiple_registers(data: &[u8], register_bank: &Arc<ModbusRegisterBank>) -> ModbusResult<Vec<u8>> {
        if data.len() < 5 {
            return Err(ModbusError::frame("Invalid write multiple registers request"));
        }
        
        let address = u16::from_be_bytes([data[0], data[1]]);
        let quantity = u16::from_be_bytes([data[2], data[3]]);
        let byte_count = data[4] as usize;
        
        if data.len() < 5 + byte_count || byte_count != (quantity as usize * 2) {
            return Err(ModbusError::frame("Incomplete write multiple registers request"));
        }
        
        // Extract register values
        let mut registers = Vec::with_capacity(quantity as usize);
        for i in 0..quantity {
            let offset = 5 + (i as usize * 2);
            let value = u16::from_be_bytes([data[offset], data[offset + 1]]);
            registers.push(value);
        }
        
        register_bank.write_multiple_registers(address, &registers)?;
        
        // Return address and quantity
        Ok(data[0..4].to_vec())
    }
    
    /// Create error response
    fn create_error_response(request: &[u8], exception_code: u8) -> ModbusResult<Vec<u8>> {
        if request.len() < MBAP_HEADER_SIZE + 2 {
            return Err(ModbusError::frame("Request too short for error response"));
        }
        
        let transaction_id = u16::from_be_bytes([request[0], request[1]]);
        let protocol_id = 0u16;
        let length = 3u16; // unit_id + function_code + exception_code
        let unit_id = request[6];
        let function_code = request[7] | 0x80; // Set exception bit
        
        let mut response = Vec::with_capacity(MBAP_HEADER_SIZE + 3);
        
        // MBAP header
        response.extend_from_slice(&transaction_id.to_be_bytes());
        response.extend_from_slice(&protocol_id.to_be_bytes());
        response.extend_from_slice(&length.to_be_bytes());
        
        // Exception PDU
        response.push(unit_id);
        response.push(function_code);
        response.push(exception_code);
        
        Ok(response)
    }
}

#[async_trait]
impl ModbusServer for ModbusTcpServer {
    async fn start(&mut self) -> ModbusResult<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Err(ModbusError::protocol("Server is already running"));
        }
        
        info!("🚀 Starting Modbus TCP server on {}", self.config.bind_address);
        
        let listener = TcpListener::bind(self.config.bind_address).await
            .map_err(|e| ModbusError::connection(format!("Failed to bind to {}: {}", self.config.bind_address, e)))?;
        
        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());
        self.start_time = Some(std::time::Instant::now());
        
        *is_running = true;
        drop(is_running);
        
        info!("✅ Modbus TCP server started successfully");
        info!("📊 Server configuration:");
        info!("   - Bind address: {}", self.config.bind_address);
        info!("   - Max connections: {}", self.config.max_connections);
        info!("   - Request timeout: {:?}", self.config.request_timeout);
        
        let register_bank = self.register_bank.clone();
        let stats = self.stats.clone();
        let request_timeout = self.config.request_timeout;
        let is_running_flag = self.is_running.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                debug!("Accepted connection from {}", addr);
                                
                                let register_bank = register_bank.clone();
                                let stats = stats.clone();
                                let shutdown_rx = shutdown_tx.subscribe();
                                
                                tokio::spawn(async move {
                                    Self::handle_client(stream, register_bank, stats, shutdown_rx, request_timeout).await;
                                });
                            }
                            Err(e) => {
                                error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Shutdown signal received, stopping server");
                        break;
                    }
                }
            }
            
            let mut is_running = is_running_flag.lock().await;
            *is_running = false;
        });
        
        Ok(())
    }
    
    async fn stop(&mut self) -> ModbusResult<()> {
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }
        
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        
        info!("⏹️  Modbus TCP server stopped");
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        // Note: This is a synchronous method, so we can't use async lock
        // In a real implementation, you might want to use a different approach
        false // Placeholder
    }
    
    fn get_stats(&self) -> ServerStats {
        // Note: This is a synchronous method, so we can't use async lock
        // In a real implementation, you might want to use a different approach
        let mut stats = ServerStats::default();
        
        if let Some(start_time) = self.start_time {
            stats.uptime_seconds = start_time.elapsed().as_secs();
        }
        
        stats.register_bank_stats = Some(self.register_bank.get_stats());
        stats
    }
    
    fn get_register_bank(&self) -> Option<Arc<ModbusRegisterBank>> {
        Some(self.register_bank.clone())
    }
}

/// Modbus RTU server (placeholder for future implementation)
pub struct ModbusRtuServer {
    // RTU server implementation fields would go here
}

impl ModbusRtuServer {
    /// Create a new RTU server
    pub fn new(_port: &str, _baud_rate: u32) -> ModbusResult<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl ModbusServer for ModbusRtuServer {
    async fn start(&mut self) -> ModbusResult<()> {
        Err(ModbusError::protocol("RTU server not implemented yet"))
    }
    
    async fn stop(&mut self) -> ModbusResult<()> {
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        false
    }
    
    fn get_stats(&self) -> ServerStats {
        ServerStats::default()
    }
    
    fn get_register_bank(&self) -> Option<Arc<ModbusRegisterBank>> {
        None
    }
} 