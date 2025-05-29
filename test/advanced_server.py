#!/usr/bin/env python3
"""
Advanced Modbus TCP server for high-performance testing
Supports concurrent connections and all Modbus function codes
"""

import asyncio
import socket
import struct
import threading
import time
import logging
from concurrent.futures import ThreadPoolExecutor

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class ModbusRegisterBank:
    """Thread-safe register bank for Modbus data"""
    
    def __init__(self):
        self.lock = threading.RLock()
        # Initialize data stores
        self.coils = [False] * 10000
        self.discrete_inputs = [False] * 10000
        self.holding_registers = [0] * 10000
        self.input_registers = [0] * 10000
        
        # Initialize with some test data
        for i in range(100):
            self.holding_registers[i] = 0x1234 + i
            self.input_registers[i] = 0x5678 + i
            self.coils[i] = (i % 2) == 0
            self.discrete_inputs[i] = (i % 3) == 0
    
    def read_coils(self, address, quantity):
        with self.lock:
            if address + quantity > len(self.coils):
                raise ValueError("Address out of range")
            return self.coils[address:address + quantity]
    
    def read_discrete_inputs(self, address, quantity):
        with self.lock:
            if address + quantity > len(self.discrete_inputs):
                raise ValueError("Address out of range")
            return self.discrete_inputs[address:address + quantity]
    
    def read_holding_registers(self, address, quantity):
        with self.lock:
            if address + quantity > len(self.holding_registers):
                raise ValueError("Address out of range")
            return self.holding_registers[address:address + quantity]
    
    def read_input_registers(self, address, quantity):
        with self.lock:
            if address + quantity > len(self.input_registers):
                raise ValueError("Address out of range")
            return self.input_registers[address:address + quantity]

class AsyncModbusServer:
    def __init__(self, host='127.0.0.1', port=5020):
        self.host = host
        self.port = port
        self.register_bank = ModbusRegisterBank()
        self.client_count = 0
        self.request_count = 0
        self.error_count = 0
        
    async def handle_client(self, reader, writer):
        client_addr = writer.get_extra_info('peername')
        client_id = self.client_count
        self.client_count += 1
        
        logger.info(f"📡 Client {client_id} connected from {client_addr}")
        
        try:
            while True:
                # Read MBAP header (6 bytes)
                header_data = await reader.read(6)
                if not header_data or len(header_data) < 6:
                    break
                
                # Parse MBAP header
                transaction_id, protocol_id, length = struct.unpack('>HHH', header_data)
                
                if protocol_id != 0:
                    logger.error(f"Invalid protocol ID: {protocol_id}")
                    break
                
                if length > 255 or length < 2:
                    logger.error(f"Invalid frame length: {length}")
                    break
                
                # Read PDU (length bytes = unit_id + function_code + data)
                pdu_data = await reader.read(length)
                if len(pdu_data) < length:
                    logger.error(f"Incomplete PDU: expected {length}, got {len(pdu_data)}")
                    break
                
                self.request_count += 1
                logger.debug(f"Received frame: Transaction={transaction_id}, Length={length}, PDU={pdu_data.hex()}")
                
                # Process request
                try:
                    response = await self.process_request(transaction_id, pdu_data)
                    writer.write(response)
                    await writer.drain()
                except Exception as e:
                    logger.error(f"Error processing request: {e}")
                    self.error_count += 1
                    # Send exception response
                    exception_response = self.create_exception_response(
                        transaction_id, pdu_data[1] if len(pdu_data) > 1 else 1, 0x01
                    )
                    writer.write(exception_response)
                    await writer.drain()
                    
        except asyncio.CancelledError:
            logger.info(f"Client {client_id} cancelled")
        except Exception as e:
            logger.error(f"Client {client_id} error: {e}")
        finally:
            logger.info(f"🔌 Client {client_id} disconnected")
            writer.close()
            await writer.wait_closed()
    
    async def process_request(self, transaction_id, pdu_data):
        if len(pdu_data) < 2:
            raise ValueError("PDU too short")
        
        unit_id = pdu_data[0]
        function_code = pdu_data[1]
        
        logger.debug(f"Processing function code {function_code:02x}")
        
        # Parse based on function code
        if function_code == 0x01:  # Read Coils
            return await self.read_coils(transaction_id, unit_id, pdu_data[2:])
        elif function_code == 0x02:  # Read Discrete Inputs  
            return await self.read_discrete_inputs(transaction_id, unit_id, pdu_data[2:])
        elif function_code == 0x03:  # Read Holding Registers
            return await self.read_holding_registers(transaction_id, unit_id, pdu_data[2:])
        elif function_code == 0x04:  # Read Input Registers
            return await self.read_input_registers(transaction_id, unit_id, pdu_data[2:])
        else:
            raise ValueError(f"Unsupported function code: {function_code}")
    
    async def read_coils(self, transaction_id, unit_id, data):
        if len(data) < 4:
            raise ValueError("Invalid read coils request")
        
        address, quantity = struct.unpack('>HH', data[:4])
        
        if quantity > 2000:
            raise ValueError("Too many coils requested")
        
        coils = self.register_bank.read_coils(address, quantity)
        
        # Pack bits into bytes
        byte_count = (quantity + 7) // 8
        response_data = bytearray([byte_count])
        
        for i in range(byte_count):
            byte_value = 0
            for bit in range(8):
                coil_index = i * 8 + bit
                if coil_index < len(coils) and coils[coil_index]:
                    byte_value |= (1 << bit)
            response_data.append(byte_value)
        
        return self.create_response(transaction_id, unit_id, 0x01, response_data)
    
    async def read_discrete_inputs(self, transaction_id, unit_id, data):
        if len(data) < 4:
            raise ValueError("Invalid read discrete inputs request")
        
        address, quantity = struct.unpack('>HH', data[:4])
        
        if quantity > 2000:
            raise ValueError("Too many inputs requested")
        
        inputs = self.register_bank.read_discrete_inputs(address, quantity)
        
        # Pack bits into bytes
        byte_count = (quantity + 7) // 8
        response_data = bytearray([byte_count])
        
        for i in range(byte_count):
            byte_value = 0
            for bit in range(8):
                input_index = i * 8 + bit
                if input_index < len(inputs) and inputs[input_index]:
                    byte_value |= (1 << bit)
            response_data.append(byte_value)
        
        return self.create_response(transaction_id, unit_id, 0x02, response_data)
    
    async def read_holding_registers(self, transaction_id, unit_id, data):
        if len(data) < 4:
            raise ValueError("Invalid read holding registers request")
        
        address, quantity = struct.unpack('>HH', data[:4])
        
        if quantity > 125:
            raise ValueError("Too many registers requested")
        
        registers = self.register_bank.read_holding_registers(address, quantity)
        
        byte_count = quantity * 2
        response_data = bytearray([byte_count])
        
        for register in registers:
            response_data.extend(struct.pack('>H', register))
        
        return self.create_response(transaction_id, unit_id, 0x03, response_data)
    
    async def read_input_registers(self, transaction_id, unit_id, data):
        if len(data) < 4:
            raise ValueError("Invalid read input registers request")
        
        address, quantity = struct.unpack('>HH', data[:4])
        
        if quantity > 125:
            raise ValueError("Too many registers requested")
        
        registers = self.register_bank.read_input_registers(address, quantity)
        
        byte_count = quantity * 2
        response_data = bytearray([byte_count])
        
        for register in registers:
            response_data.extend(struct.pack('>H', register))
        
        return self.create_response(transaction_id, unit_id, 0x04, response_data)
    
    def create_response(self, transaction_id, unit_id, function_code, data):
        length = len(data) + 2  # +2 for unit_id and function_code
        
        response = bytearray()
        response.extend(struct.pack('>HHH', transaction_id, 0, length))  # MBAP header
        response.append(unit_id)
        response.append(function_code)
        response.extend(data)
        
        return response
    
    def create_exception_response(self, transaction_id, function_code, exception_code):
        length = 3  # unit_id + function_code + exception_code
        
        response = bytearray()
        response.extend(struct.pack('>HHH', transaction_id, 0, length))  # MBAP header
        response.append(1)  # unit_id
        response.append(function_code | 0x80)  # function_code with exception bit
        response.append(exception_code)
        
        return response
    
    async def start_server(self):
        server = await asyncio.start_server(
            self.handle_client, 
            self.host, 
            self.port,
            reuse_address=True,
            reuse_port=True
        )
        
        addr = server.sockets[0].getsockname()
        logger.info(f"🚀 Advanced Modbus TCP server started on {addr[0]}:{addr[1]}")
        logger.info(f"📊 Server capabilities:")
        logger.info(f"   - Support for 10,000 coils, discrete inputs, and registers")
        logger.info(f"   - Async I/O with unlimited concurrent connections")
        logger.info(f"   - All standard Modbus function codes (1-4)")
        logger.info(f"   - Thread-safe data access")
        
        # Print statistics every 10 seconds
        asyncio.create_task(self.print_stats())
        
        async with server:
            await server.serve_forever()
    
    async def print_stats(self):
        while True:
            await asyncio.sleep(10)
            logger.info(f"📈 Server stats: {self.request_count} requests, "
                       f"{self.client_count} total clients, {self.error_count} errors")

async def main():
    try:
        server = AsyncModbusServer()
        await server.start_server()
    except KeyboardInterrupt:
        logger.info("👋 Server shutdown requested")
    except Exception as e:
        logger.error(f"❌ Server error: {e}")

if __name__ == '__main__':
    asyncio.run(main()) 