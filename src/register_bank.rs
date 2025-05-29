/// Modbus register bank for server-side data storage
/// 
/// This module provides thread-safe storage for Modbus data including coils,
/// discrete inputs, holding registers, and input registers.

use std::sync::{Arc, RwLock};
use crate::error::{ModbusError, ModbusResult};

/// Default register bank size
const DEFAULT_COILS_SIZE: usize = 10000;
const DEFAULT_DISCRETE_INPUTS_SIZE: usize = 10000;
const DEFAULT_HOLDING_REGISTERS_SIZE: usize = 10000;
const DEFAULT_INPUT_REGISTERS_SIZE: usize = 10000;

/// Thread-safe register bank for Modbus data
#[derive(Debug, Clone)]
pub struct ModbusRegisterBank {
    coils: Arc<RwLock<Vec<bool>>>,
    discrete_inputs: Arc<RwLock<Vec<bool>>>,
    holding_registers: Arc<RwLock<Vec<u16>>>,
    input_registers: Arc<RwLock<Vec<u16>>>,
}

impl ModbusRegisterBank {
    /// Create a new register bank with default sizes
    pub fn new() -> Self {
        Self::with_sizes(
            DEFAULT_COILS_SIZE,
            DEFAULT_DISCRETE_INPUTS_SIZE,
            DEFAULT_HOLDING_REGISTERS_SIZE,
            DEFAULT_INPUT_REGISTERS_SIZE,
        )
    }
    
    /// Create a new register bank with custom sizes
    pub fn with_sizes(
        coils_size: usize,
        discrete_inputs_size: usize,
        holding_registers_size: usize,
        input_registers_size: usize,
    ) -> Self {
        let mut coils = vec![false; coils_size];
        let mut discrete_inputs = vec![false; discrete_inputs_size];
        let mut holding_registers = vec![0u16; holding_registers_size];
        let mut input_registers = vec![0u16; input_registers_size];
        
        // Initialize with some test data
        for i in 0..(coils_size.min(100)) {
            coils[i] = (i % 2) == 0;
        }
        
        for i in 0..(discrete_inputs_size.min(100)) {
            discrete_inputs[i] = (i % 3) == 0;
        }
        
        for i in 0..(holding_registers_size.min(100)) {
            holding_registers[i] = 0x1234 + i as u16;
        }
        
        for i in 0..(input_registers_size.min(100)) {
            input_registers[i] = 0x5678 + i as u16;
        }
        
        Self {
            coils: Arc::new(RwLock::new(coils)),
            discrete_inputs: Arc::new(RwLock::new(discrete_inputs)),
            holding_registers: Arc::new(RwLock::new(holding_registers)),
            input_registers: Arc::new(RwLock::new(input_registers)),
        }
    }
    
    /// Read coils
    pub fn read_coils(&self, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        let coils = self.coils.read().unwrap();
        let start = address as usize;
        let end = start + quantity as usize;
        
        if end > coils.len() {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        Ok(coils[start..end].to_vec())
    }
    
    /// Write single coil
    pub fn write_single_coil(&self, address: u16, value: bool) -> ModbusResult<()> {
        let mut coils = self.coils.write().unwrap();
        let addr = address as usize;
        
        if addr >= coils.len() {
            return Err(ModbusError::invalid_address(address, 1));
        }
        
        coils[addr] = value;
        Ok(())
    }
    
    /// Write multiple coils
    pub fn write_multiple_coils(&self, address: u16, values: &[bool]) -> ModbusResult<()> {
        let mut coils = self.coils.write().unwrap();
        let start = address as usize;
        let end = start + values.len();
        
        if end > coils.len() {
            return Err(ModbusError::invalid_address(address, values.len() as u16));
        }
        
        coils[start..end].copy_from_slice(values);
        Ok(())
    }
    
    /// Read discrete inputs
    pub fn read_discrete_inputs(&self, address: u16, quantity: u16) -> ModbusResult<Vec<bool>> {
        let inputs = self.discrete_inputs.read().unwrap();
        let start = address as usize;
        let end = start + quantity as usize;
        
        if end > inputs.len() {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        Ok(inputs[start..end].to_vec())
    }
    
    /// Read holding registers
    pub fn read_holding_registers(&self, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        let registers = self.holding_registers.read().unwrap();
        let start = address as usize;
        let end = start + quantity as usize;
        
        if end > registers.len() {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        Ok(registers[start..end].to_vec())
    }
    
    /// Write single holding register
    pub fn write_single_register(&self, address: u16, value: u16) -> ModbusResult<()> {
        let mut registers = self.holding_registers.write().unwrap();
        let addr = address as usize;
        
        if addr >= registers.len() {
            return Err(ModbusError::invalid_address(address, 1));
        }
        
        registers[addr] = value;
        Ok(())
    }
    
    /// Write multiple holding registers
    pub fn write_multiple_registers(&self, address: u16, values: &[u16]) -> ModbusResult<()> {
        let mut registers = self.holding_registers.write().unwrap();
        let start = address as usize;
        let end = start + values.len();
        
        if end > registers.len() {
            return Err(ModbusError::invalid_address(address, values.len() as u16));
        }
        
        registers[start..end].copy_from_slice(values);
        Ok(())
    }
    
    /// Read input registers
    pub fn read_input_registers(&self, address: u16, quantity: u16) -> ModbusResult<Vec<u16>> {
        let registers = self.input_registers.read().unwrap();
        let start = address as usize;
        let end = start + quantity as usize;
        
        if end > registers.len() {
            return Err(ModbusError::invalid_address(address, quantity));
        }
        
        Ok(registers[start..end].to_vec())
    }
    
    /// Set input register value (for simulation)
    pub fn set_input_register(&self, address: u16, value: u16) -> ModbusResult<()> {
        let mut registers = self.input_registers.write().unwrap();
        let addr = address as usize;
        
        if addr >= registers.len() {
            return Err(ModbusError::invalid_address(address, 1));
        }
        
        registers[addr] = value;
        Ok(())
    }
    
    /// Set discrete input value (for simulation)
    pub fn set_discrete_input(&self, address: u16, value: bool) -> ModbusResult<()> {
        let mut inputs = self.discrete_inputs.write().unwrap();
        let addr = address as usize;
        
        if addr >= inputs.len() {
            return Err(ModbusError::invalid_address(address, 1));
        }
        
        inputs[addr] = value;
        Ok(())
    }
    
    /// Get register bank statistics
    pub fn get_stats(&self) -> RegisterBankStats {
        RegisterBankStats {
            coils_count: self.coils.read().unwrap().len(),
            discrete_inputs_count: self.discrete_inputs.read().unwrap().len(),
            holding_registers_count: self.holding_registers.read().unwrap().len(),
            input_registers_count: self.input_registers.read().unwrap().len(),
        }
    }
}

impl Default for ModbusRegisterBank {
    fn default() -> Self {
        Self::new()
    }
}

/// Register bank statistics
#[derive(Debug, Clone)]
pub struct RegisterBankStats {
    pub coils_count: usize,
    pub discrete_inputs_count: usize,
    pub holding_registers_count: usize,
    pub input_registers_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_bank_creation() {
        let bank = ModbusRegisterBank::new();
        let stats = bank.get_stats();
        
        assert_eq!(stats.coils_count, DEFAULT_COILS_SIZE);
        assert_eq!(stats.holding_registers_count, DEFAULT_HOLDING_REGISTERS_SIZE);
    }
    
    #[test]
    fn test_read_write_coils() {
        let bank = ModbusRegisterBank::new();
        
        // Write single coil
        bank.write_single_coil(10, true).unwrap();
        
        // Read coils
        let coils = bank.read_coils(10, 1).unwrap();
        assert_eq!(coils[0], true);
        
        // Write multiple coils
        bank.write_multiple_coils(20, &[true, false, true]).unwrap();
        let coils = bank.read_coils(20, 3).unwrap();
        assert_eq!(coils, vec![true, false, true]);
    }
    
    #[test]
    fn test_read_write_registers() {
        let bank = ModbusRegisterBank::new();
        
        // Write single register
        bank.write_single_register(5, 0xABCD).unwrap();
        
        // Read registers
        let registers = bank.read_holding_registers(5, 1).unwrap();
        assert_eq!(registers[0], 0xABCD);
        
        // Write multiple registers
        bank.write_multiple_registers(100, &[0x1111, 0x2222, 0x3333]).unwrap();
        let registers = bank.read_holding_registers(100, 3).unwrap();
        assert_eq!(registers, vec![0x1111, 0x2222, 0x3333]);
    }
} 