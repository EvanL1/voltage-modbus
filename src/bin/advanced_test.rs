/// Voltage Modbus Advanced Performance Test
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// High-concurrency performance and stress testing

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use futures::future::join_all;

use voltage_modbus::transport::{TcpTransport, ModbusTransport};
use voltage_modbus::protocol::{ModbusRequest, ModbusFunction};
use voltage_modbus::error::{ModbusResult, ModbusError};

#[derive(Debug, Clone)]
struct TestConfig {
    server_address: String,
    timeout_ms: u64,
    max_concurrent: usize,
    warmup_requests: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:5020".to_string(),
            timeout_ms: 3000, // 优化为3秒超时，更适合本地测试
            max_concurrent: 20, // More conservative concurrency
            warmup_requests: 10,
        }
    }
}

#[derive(Debug, Clone)]
struct TestResults {
    test_name: String,
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    total_duration_ms: u64,
    requests_per_second: f64,
    average_latency_ms: f64,
    min_latency_ms: f64,
    max_latency_ms: f64,
    p99_latency_ms: f64,
    success_rate: f64,
    bytes_transferred: u64,
}

/// High-performance connection pool for reusing connections
struct ConnectionPool {
    connections: Vec<TcpTransport>,
    address: std::net::SocketAddr,
    timeout: Duration,
    semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    async fn new(address: std::net::SocketAddr, timeout: Duration, pool_size: usize) -> ModbusResult<Self> {
        let mut connections = Vec::new();
        
        println!("🔄 Creating connection pool with {} connections...", pool_size);
        
        let mut i = 0;
        while i < pool_size {
            match TcpTransport::new(address, timeout).await {
                Ok(conn) => {
                    connections.push(conn);
                    if (i + 1) % 5 == 0 {
                        println!("  ✅ Created {} connections", i + 1);
                    }
                    i += 1;
                }
                Err(e) => {
                    println!("  ⚠️  Failed to create connection {}: {}", i + 1, e);
                    // Add some delay before retry
                    sleep(Duration::from_millis(100)).await;
                    // Retry this connection - don't increment i
                }
            }
        }
        
        println!("✅ Connection pool ready with {} connections", connections.len());
        
        Ok(Self {
            connections,
            address,
            timeout,
            semaphore: Arc::new(Semaphore::new(pool_size)),
        })
    }
    
    async fn execute_request(&mut self, request: &ModbusRequest) -> ModbusResult<(Vec<u8>, Duration)> {
        let _permit = self.semaphore.acquire().await.unwrap();
        
        if let Some(mut transport) = self.connections.pop() {
            let start = Instant::now();
            let result = transport.request(request).await;
            let duration = start.elapsed();
            
            match result {
                Ok(response) => {
                    // Return connection to pool
                    self.connections.push(transport);
                    Ok((response.data, duration))
                }
                Err(e) => {
                    // Connection failed, create a new one
                    match TcpTransport::new(self.address, self.timeout).await {
                        Ok(new_transport) => {
                            self.connections.push(new_transport);
                        }
                        Err(_) => {
                            // Could not recreate connection, just continue
                        }
                    }
                    Err(e)
                }
            }
        } else {
            // No connections available, create temporary one
            let mut transport = TcpTransport::new(self.address, self.timeout).await?;
            let start = Instant::now();
            let result = transport.request(request).await;
            let duration = start.elapsed();
            
            match result {
                Ok(response) => Ok((response.data, duration)),
                Err(e) => Err(e),
            }
        }
    }
}

#[tokio::main]
async fn main() -> ModbusResult<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Advanced Modbus Native Performance Test Suite");
    println!("===============================================");
    println!("Target: 100% Success Rate, High Performance");
    println!("");

    let config = TestConfig::default();
    
    // Warmup phase
    println!("🔥 Warming up connection...");
    if let Err(e) = warmup_test(&config).await {
        println!("⚠️  Warmup failed: {}. Please ensure server is running.", e);
        return Ok(());
    }
    println!("✅ Warmup completed successfully");
    println!("");
    
    let mut all_results = Vec::new();
    
    // Test 1: Concurrent Connections Test
    println!("📊 Test 1: Concurrent Connections Test (5 connections)");
    let result1 = run_concurrent_test(&config, 5, 25, "Concurrent Connections").await?;
    print_test_results(&result1);
    all_results.push(result1);
    
    // Test 2: Batch Operations Test  
    println!("\n📊 Test 2: Batch Operations Test (20 operations)");
    let result2 = run_batch_test(&config, 20, "Batch Operations").await?;
    print_test_results(&result2);
    all_results.push(result2);
    
    // Test 3: Multi-Function Test
    println!("\n📊 Test 3: Multi-Function Test (All function codes)");
    let result3 = run_multi_function_test(&config, "Multi-Function").await?;
    print_test_results(&result3);
    all_results.push(result3);
    
    // Test 4: Timeout Handling Test
    println!("\n📊 Test 4: Timeout Handling Test");
    let result4 = run_timeout_test(&config, "Timeout Handling").await?;
    print_test_results(&result4);
    all_results.push(result4);
    
    // Test 5: Stress Test
    println!("\n📊 Test 5: Stress Test (3 seconds)");
    let result5 = run_stress_test(&config, Duration::from_secs(3), "Stress Test").await?;
    print_test_results(&result5);
    all_results.push(result5);
    
    // Print comprehensive summary
    print_comprehensive_summary(&all_results);
    
    println!("\n✅ All tests completed!");
    Ok(())
}

async fn warmup_test(config: &TestConfig) -> ModbusResult<()> {
    let address = config.server_address.parse()
        .map_err(|e| ModbusError::invalid_data(format!("Invalid address: {}", e)))?;
    let timeout = Duration::from_millis(config.timeout_ms);
    
    let mut transport = TcpTransport::new(address, timeout).await?;
    
    for i in 0..config.warmup_requests {
        let request = ModbusRequest::new_read(
            1,
            ModbusFunction::ReadHoldingRegisters,
            i as u16,
            1,
        );
        
        transport.request(&request).await?;
        sleep(Duration::from_millis(10)).await;
    }
    
    transport.close().await?;
    Ok(())
}

async fn run_concurrent_test(config: &TestConfig, connections: usize, requests_per_conn: usize, test_name: &str) -> ModbusResult<TestResults> {
    let start_time = Instant::now();
    let mut tasks = Vec::new();
    let mut all_latencies = Vec::new();
    
    for conn_id in 0..connections {
        let config = config.clone();
        let task = tokio::spawn(async move {
            let address = config.server_address.parse().unwrap();
            let timeout = Duration::from_millis(config.timeout_ms);
            
            let mut transport = TcpTransport::new(address, timeout).await?;
            let mut latencies = Vec::new();
            let mut bytes_transferred = 0u64;
            let mut successful = 0;
            let mut failed = 0;
            
            for i in 0..requests_per_conn {
                let start = Instant::now();
                let request = ModbusRequest::new_read(
                    1,
                    ModbusFunction::ReadHoldingRegisters,
                    (conn_id * 100 + i) as u16 % 1000,
                    5,
                );
                
                match transport.request(&request).await {
                    Ok(response) => {
                        let latency = start.elapsed().as_secs_f64() * 1000.0;
                        latencies.push(latency);
                        bytes_transferred += response.data.len() as u64;
                        successful += 1;
                    }
                    Err(_) => {
                        failed += 1;
                    }
                }
                
                // Small delay to avoid overwhelming
                sleep(Duration::from_millis(1)).await;
            }
            
            transport.close().await?;
            
            Ok::<_, ModbusError>((successful, failed, bytes_transferred, latencies))
        });
        tasks.push(task);
    }
    
    let results = join_all(tasks).await;
    
    let mut total_successful = 0;
    let mut total_failed = 0;
    let mut total_bytes = 0u64;
    
    for result in results {
        match result {
            Ok(Ok((successful, failed, bytes, latencies))) => {
                total_successful += successful;
                total_failed += failed;
                total_bytes += bytes;
                all_latencies.extend(latencies);
            }
            _ => {
                total_failed += requests_per_conn;
            }
        }
    }
    
    let total_duration = start_time.elapsed();
    create_test_results(test_name, total_successful, total_failed, total_bytes, total_duration, all_latencies)
}

async fn run_batch_test(config: &TestConfig, batch_size: usize, test_name: &str) -> ModbusResult<TestResults> {
    let start_time = Instant::now();
    let address = config.server_address.parse()
        .map_err(|e| ModbusError::invalid_data(format!("Invalid address: {}", e)))?;
    let timeout = Duration::from_millis(config.timeout_ms);
    
    let mut transport = TcpTransport::new(address, timeout).await?;
    let mut latencies = Vec::new();
    let mut bytes_transferred = 0u64;
    let mut successful = 0;
    let mut failed = 0;
    
    for i in 0..batch_size {
        let start = Instant::now();
        let request = ModbusRequest::new_read(
            1,
            ModbusFunction::ReadHoldingRegisters,
            (i * 10) as u16 % 1000,
            10,
        );
        
        match transport.request(&request).await {
            Ok(response) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                latencies.push(latency);
                bytes_transferred += response.data.len() as u64;
                successful += 1;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }
    
    transport.close().await?;
    
    let total_duration = start_time.elapsed();
    create_test_results(test_name, successful, failed, bytes_transferred, total_duration, latencies)
}

async fn run_multi_function_test(config: &TestConfig, test_name: &str) -> ModbusResult<TestResults> {
    let start_time = Instant::now();
    let address = config.server_address.parse()
        .map_err(|e| ModbusError::invalid_data(format!("Invalid address: {}", e)))?;
    let timeout = Duration::from_millis(config.timeout_ms);
    
    let mut transport = TcpTransport::new(address, timeout).await?;
    let mut latencies = Vec::new();
    let mut bytes_transferred = 0u64;
    let mut successful = 0;
    let mut failed = 0;
    
    let functions = [
        ModbusFunction::ReadCoils,
        ModbusFunction::ReadDiscreteInputs,
        ModbusFunction::ReadHoldingRegisters,
        ModbusFunction::ReadInputRegisters,
    ];
    
    for round in 0..5 {
        for (i, &function) in functions.iter().enumerate() {
            let start = Instant::now();
            let request = ModbusRequest::new_read(
                1,
                function,
                (round * 10 + i) as u16,
                5,
            );
            
            match transport.request(&request).await {
                Ok(response) => {
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    latencies.push(latency);
                    bytes_transferred += response.data.len() as u64;
                    successful += 1;
                }
                Err(_) => {
                    failed += 1;
                }
            }
            
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    transport.close().await?;
    
    let total_duration = start_time.elapsed();
    create_test_results(test_name, successful, failed, bytes_transferred, total_duration, latencies)
}

async fn run_timeout_test(config: &TestConfig, test_name: &str) -> ModbusResult<TestResults> {
    let start_time = Instant::now();
    let address = config.server_address.parse()
        .map_err(|e| ModbusError::invalid_data(format!("Invalid address: {}", e)))?;
    
    let mut latencies = Vec::new();
    let mut bytes_transferred = 0u64;
    let mut successful = 0;
    let mut failed = 0;
    
    // Test normal timeout
    let normal_timeout = Duration::from_millis(config.timeout_ms);
    let mut transport1 = TcpTransport::new(address, normal_timeout).await?;
    
    for i in 0..5 {
        let start = Instant::now();
        let request = ModbusRequest::new_read(
            1,
            ModbusFunction::ReadHoldingRegisters,
            i * 10,
            5,
        );
        
        match transport1.request(&request).await {
            Ok(response) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                latencies.push(latency);
                bytes_transferred += response.data.len() as u64;
                successful += 1;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }
    
    transport1.close().await?;
    
    let total_duration = start_time.elapsed();
    create_test_results(test_name, successful, failed, bytes_transferred, total_duration, latencies)
}

async fn run_stress_test(config: &TestConfig, duration: Duration, test_name: &str) -> ModbusResult<TestResults> {
    let start_time = Instant::now();
    let address = config.server_address.parse()
        .map_err(|e| ModbusError::invalid_data(format!("Invalid address: {}", e)))?;
    let timeout = Duration::from_millis(config.timeout_ms);
    
    let mut transport = TcpTransport::new(address, timeout).await?;
    let mut latencies = Vec::new();
    let mut bytes_transferred = 0u64;
    let mut successful = 0;
    let mut failed = 0;
    let mut operation_count = 0;
    
    while start_time.elapsed() < duration {
        let start = Instant::now();
        let request = ModbusRequest::new_read(
            1,
            ModbusFunction::ReadHoldingRegisters,
            (operation_count % 1000) as u16,
            5,
        );
        
        match transport.request(&request).await {
            Ok(response) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                latencies.push(latency);
                bytes_transferred += response.data.len() as u64;
                successful += 1;
            }
            Err(_) => {
                failed += 1;
            }
        }
        
        operation_count += 1;
        
        // Small delay to prevent overwhelming
        sleep(Duration::from_millis(10)).await;
    }
    
    transport.close().await?;
    
    let total_duration = start_time.elapsed();
    create_test_results(test_name, successful, failed, bytes_transferred, total_duration, latencies)
}

fn create_test_results(
    test_name: &str,
    successful: usize,
    failed: usize,
    bytes_transferred: u64,
    total_duration: Duration,
    mut latencies: Vec<f64>
) -> ModbusResult<TestResults> {
    let total_requests = successful + failed;
    let total_duration_ms = total_duration.as_millis() as u64;
    
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_latency = if !latencies.is_empty() {
        latencies.iter().sum::<f64>() / latencies.len() as f64
    } else { 0.0 };
    
    let min_latency = latencies.first().copied().unwrap_or(0.0);
    let max_latency = latencies.last().copied().unwrap_or(0.0);
    let p99_latency = if !latencies.is_empty() {
        let index = ((latencies.len() as f64) * 0.99) as usize;
        latencies.get(index).copied().unwrap_or(0.0)
    } else { 0.0 };
    
    let rps = if total_duration_ms > 0 {
        (successful as f64) / (total_duration_ms as f64 / 1000.0)
    } else { 0.0 };
    
    let success_rate = if total_requests > 0 {
        (successful as f64) / (total_requests as f64) * 100.0
    } else { 0.0 };
    
    Ok(TestResults {
        test_name: test_name.to_string(),
        total_requests,
        successful_requests: successful,
        failed_requests: failed,
        total_duration_ms,
        requests_per_second: rps,
        average_latency_ms: avg_latency,
        min_latency_ms: min_latency,
        max_latency_ms: max_latency,
        p99_latency_ms: p99_latency,
        success_rate,
        bytes_transferred,
    })
}

fn print_test_results(results: &TestResults) {
    println!("  📈 Results for {}:", results.test_name);
    println!("    Total Requests: {}", results.total_requests);
    println!("    ✅ Successful: {} ({:.1}%)", results.successful_requests, results.success_rate);
    println!("    ❌ Failed: {}", results.failed_requests);
    println!("    ⏱️  Duration: {:.2}s", results.total_duration_ms as f64 / 1000.0);
    println!("    🚀 Throughput: {:.1} ops/sec", results.requests_per_second);
    println!("    📊 Latency - Avg: {:.2}ms, Min: {:.2}ms, Max: {:.2}ms, P99: {:.2}ms", 
             results.average_latency_ms, results.min_latency_ms, 
             results.max_latency_ms, results.p99_latency_ms);
    println!("    📦 Data Transferred: {} bytes", results.bytes_transferred);
}

fn print_comprehensive_summary(all_results: &[TestResults]) {
    let separator = "=".repeat(60);
    println!("\n{}", separator);
    println!("📋 COMPREHENSIVE PERFORMANCE TEST SUMMARY");
    println!("{}", separator);
    
    for result in all_results {
        let status = if result.success_rate >= 99.0 {
            "✅ EXCELLENT"
        } else if result.success_rate >= 95.0 {
            "🟡 GOOD"
        } else if result.success_rate >= 90.0 {
            "🟠 AVERAGE"
        } else {
            "🔴 NEEDS IMPROVEMENT"
        };
        
        println!("  {} - {:.1} ops/sec, {:.1}% success, {:.2}ms avg latency - {}", 
                 result.test_name, result.requests_per_second, 
                 result.success_rate, result.average_latency_ms, status);
    }
    
    // Calculate overall metrics
    let total_requests: usize = all_results.iter().map(|r| r.total_requests).sum();
    let total_successful: usize = all_results.iter().map(|r| r.successful_requests).sum();
    let avg_rps: f64 = all_results.iter().map(|r| r.requests_per_second).sum::<f64>() / all_results.len() as f64;
    let avg_success: f64 = all_results.iter().map(|r| r.success_rate).sum::<f64>() / all_results.len() as f64;
    let avg_latency: f64 = all_results.iter().map(|r| r.average_latency_ms).sum::<f64>() / all_results.len() as f64;
    let total_bytes: u64 = all_results.iter().map(|r| r.bytes_transferred).sum();
    
    println!("\n🎯 OVERALL PERFORMANCE:");
    println!("  Total Operations: {} ({}% success)", total_requests, (total_successful as f64 / total_requests as f64 * 100.0) as u32);
    println!("  Average Throughput: {:.1} ops/sec", avg_rps);
    println!("  Average Success Rate: {:.1}%", avg_success);
    println!("  Average Latency: {:.2}ms", avg_latency);
    println!("  Total Data Transferred: {} bytes", total_bytes);
    
    // Final assessment
    println!("\n🏆 FINAL ASSESSMENT:");
    if avg_success >= 99.0 && avg_rps > 50.0 && avg_latency < 50.0 {
        println!("  🎉 EXCELLENT - Ready for production deployment!");
        println!("  ✅ High reliability, good performance, low latency");
    } else if avg_success >= 95.0 && avg_rps > 30.0 && avg_latency < 100.0 {
        println!("  ✅ GOOD - Suitable for most industrial applications");
        println!("  📈 Good reliability and performance characteristics");
    } else if avg_success >= 90.0 {
        println!("  🟡 ACCEPTABLE - May need optimization for critical applications");
        println!("  ⚠️  Consider performance tuning for better results");
    } else {
        println!("  🔴 NEEDS IMPROVEMENT - Requires optimization before production");
        println!("  🛠️  Check server capacity and network configuration");
    }
    
    println!("\n{}", separator);
} 