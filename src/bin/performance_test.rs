/// Voltage Modbus Performance Test
/// 
/// Author: Evan Liu <evan.liu@voltageenergy.com>
/// Comprehensive performance testing and benchmarking

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use futures::future::join_all;
use serde::{Serialize, Deserialize};

use voltage_modbus::transport::{TcpTransport, ModbusTransport};
use voltage_modbus::protocol::{ModbusRequest, ModbusFunction};
use voltage_modbus::error::{ModbusResult, ModbusError};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestConfig {
    server_address: String,
    timeout_ms: u64,
    concurrent_clients: usize,
    requests_per_client: usize,
    test_duration_secs: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:5020".to_string(),
            timeout_ms: 5000,
            concurrent_clients: 10,
            requests_per_client: 100,
            test_duration_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[tokio::main]
async fn main() -> ModbusResult<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Modbus Native Performance Test Suite");
    println!("=======================================");

    let config = TestConfig::default();
    
    // Run different test scenarios
    let mut all_results = Vec::new();
    
    // Test 1: Basic throughput test
    println!("\n📊 Test 1: Basic Throughput Test");
    let result1 = run_throughput_test(&config, "Basic Throughput").await?;
    print_test_results(&result1);
    all_results.push(result1);
    
    // Test 2: High concurrency test
    println!("\n📊 Test 2: High Concurrency Test");
    let mut high_concurrency_config = config.clone();
    high_concurrency_config.concurrent_clients = 50;
    high_concurrency_config.requests_per_client = 20;
    let result2 = run_throughput_test(&high_concurrency_config, "High Concurrency").await?;
    print_test_results(&result2);
    all_results.push(result2);
    
    // Test 3: Latency test
    println!("\n📊 Test 3: Latency Test");
    let mut latency_config = config.clone();
    latency_config.concurrent_clients = 1;
    latency_config.requests_per_client = 1000;
    let result3 = run_throughput_test(&latency_config, "Latency Test").await?;
    print_test_results(&result3);
    all_results.push(result3);
    
    // Test 4: Stress test
    println!("\n📊 Test 4: Stress Test");
    let result4 = run_stress_test(&config, "Stress Test").await?;
    print_test_results(&result4);
    all_results.push(result4);
    
    // Print summary
    print_summary(&all_results);
    
    println!("\n✅ All performance tests completed!");
    Ok(())
}

async fn run_throughput_test(config: &TestConfig, test_name: &str) -> ModbusResult<TestResults> {
    let start_time = Instant::now();
    let semaphore = Arc::new(Semaphore::new(config.concurrent_clients));
    let mut handles = Vec::new();
    let mut latencies = Vec::new();
    
    for client_id in 0..config.concurrent_clients {
        let config = config.clone();
        let semaphore = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            run_client_requests(client_id, &config).await
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut total_bytes = 0u64;
    
    for handle in handles {
        match handle.await {
            Ok(Ok(client_result)) => {
                total_requests += client_result.total_requests;
                successful_requests += client_result.successful_requests;
                failed_requests += client_result.failed_requests;
                total_bytes += client_result.bytes_transferred;
                latencies.extend(client_result.latencies);
            }
            Ok(Err(e)) => {
                eprintln!("Client error: {}", e);
                failed_requests += config.requests_per_client;
            }
            Err(e) => {
                eprintln!("Task error: {}", e);
                failed_requests += config.requests_per_client;
            }
        }
    }
    
    let total_duration = start_time.elapsed();
    let total_duration_ms = total_duration.as_millis() as u64;
    
    // Calculate statistics
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
        (successful_requests as f64) / (total_duration_ms as f64 / 1000.0)
    } else { 0.0 };
    
    let success_rate = if total_requests > 0 {
        (successful_requests as f64) / (total_requests as f64) * 100.0
    } else { 0.0 };
    
    Ok(TestResults {
        test_name: test_name.to_string(),
        total_requests,
        successful_requests,
        failed_requests,
        total_duration_ms,
        requests_per_second: rps,
        average_latency_ms: avg_latency,
        min_latency_ms: min_latency,
        max_latency_ms: max_latency,
        p99_latency_ms: p99_latency,
        success_rate,
        bytes_transferred: total_bytes,
    })
}

async fn run_stress_test(config: &TestConfig, test_name: &str) -> ModbusResult<TestResults> {
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(config.test_duration_secs);
    let semaphore = Arc::new(Semaphore::new(config.concurrent_clients));
    
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut total_bytes = 0u64;
    let mut latencies = Vec::new();
    
    while start_time.elapsed() < test_duration {
        let mut handles = Vec::new();
        
        for client_id in 0..config.concurrent_clients {
            let config = config.clone();
            let semaphore = semaphore.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                run_single_request(client_id, &config).await
            });
            
            handles.push(handle);
        }
        
        // Collect batch results
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => {
                    total_requests += 1;
                    if result.success {
                        successful_requests += 1;
                        total_bytes += result.bytes;
                        latencies.push(result.latency_ms);
                    } else {
                        failed_requests += 1;
                    }
                }
                _ => {
                    total_requests += 1;
                    failed_requests += 1;
                }
            }
        }
        
        // Small delay to prevent overwhelming
        sleep(Duration::from_millis(10)).await;
    }
    
    let total_duration_ms = start_time.elapsed().as_millis() as u64;
    
    // Calculate statistics
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
        (successful_requests as f64) / (total_duration_ms as f64 / 1000.0)
    } else { 0.0 };
    
    let success_rate = if total_requests > 0 {
        (successful_requests as f64) / (total_requests as f64) * 100.0
    } else { 0.0 };
    
    Ok(TestResults {
        test_name: test_name.to_string(),
        total_requests,
        successful_requests,
        failed_requests,
        total_duration_ms,
        requests_per_second: rps,
        average_latency_ms: avg_latency,
        min_latency_ms: min_latency,
        max_latency_ms: max_latency,
        p99_latency_ms: p99_latency,
        success_rate,
        bytes_transferred: total_bytes,
    })
}

#[derive(Debug)]
struct ClientResult {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    bytes_transferred: u64,
    latencies: Vec<f64>,
}

#[derive(Debug)]
struct SingleRequestResult {
    success: bool,
    bytes: u64,
    latency_ms: f64,
}

async fn run_client_requests(client_id: usize, config: &TestConfig) -> ModbusResult<ClientResult> {
    let address = config.server_address.parse()
        .map_err(|e| ModbusError::invalid_data(format!("Invalid address: {}", e)))?;
    let timeout = Duration::from_millis(config.timeout_ms);
    
    let mut transport = TcpTransport::new(address, timeout).await?;
    
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut bytes_transferred = 0u64;
    let mut latencies = Vec::new();
    
    for i in 0..config.requests_per_client {
        let request_start = Instant::now();
        
        let request = ModbusRequest::new_read(
            1, // slave_id
            ModbusFunction::ReadHoldingRegisters,
            (i % 1000) as u16, // varying address
            10, // quantity
        );
        
        match transport.request(&request).await {
            Ok(response) => {
                let latency = request_start.elapsed().as_secs_f64() * 1000.0;
                latencies.push(latency);
                successful_requests += 1;
                bytes_transferred += response.data.len() as u64;
            }
            Err(e) => {
                eprintln!("Client {} request {} failed: {:?}", client_id, i, e);
                failed_requests += 1;
            }
        }
    }
    
    transport.close().await?;
    
    Ok(ClientResult {
        total_requests: config.requests_per_client,
        successful_requests,
        failed_requests,
        bytes_transferred,
        latencies,
    })
}

async fn run_single_request(client_id: usize, config: &TestConfig) -> ModbusResult<SingleRequestResult> {
    let address = config.server_address.parse()
        .map_err(|e| ModbusError::invalid_data(format!("Invalid address: {}", e)))?;
    let timeout = Duration::from_millis(config.timeout_ms);
    
    let mut transport = TcpTransport::new(address, timeout).await?;
    let request_start = Instant::now();
    
    let request = ModbusRequest::new_read(
        1, // slave_id
        ModbusFunction::ReadHoldingRegisters,
        (client_id % 1000) as u16, // varying address
        5, // quantity
    );
    
    let result = match transport.request(&request).await {
        Ok(response) => {
            let latency = request_start.elapsed().as_secs_f64() * 1000.0;
            SingleRequestResult {
                success: true,
                bytes: response.data.len() as u64,
                latency_ms: latency,
            }
        }
        Err(_) => {
            SingleRequestResult {
                success: false,
                bytes: 0,
                latency_ms: 0.0,
            }
        }
    };
    
    transport.close().await?;
    Ok(result)
}

fn print_test_results(results: &TestResults) {
    println!("  📈 Results for {}:", results.test_name);
    println!("    Total Requests: {}", results.total_requests);
    println!("    Successful: {} ({:.1}%)", results.successful_requests, results.success_rate);
    println!("    Failed: {}", results.failed_requests);
    println!("    Duration: {:.2}s", results.total_duration_ms as f64 / 1000.0);
    println!("    Throughput: {:.1} RPS", results.requests_per_second);
    println!("    Latency - Avg: {:.2}ms, Min: {:.2}ms, Max: {:.2}ms, P99: {:.2}ms", 
             results.average_latency_ms, results.min_latency_ms, 
             results.max_latency_ms, results.p99_latency_ms);
    println!("    Data Transferred: {} bytes", results.bytes_transferred);
}

fn print_summary(all_results: &[TestResults]) {
    println!("\n📋 Performance Test Summary");
    println!("===========================");
    
    for result in all_results {
        let performance_rating = if result.requests_per_second > 1000.0 && result.success_rate > 99.0 {
            "🟢 Excellent"
        } else if result.requests_per_second > 500.0 && result.success_rate > 95.0 {
            "🟡 Good"
        } else if result.requests_per_second > 100.0 && result.success_rate > 90.0 {
            "🟠 Average"
        } else {
            "🔴 Poor"
        };
        
        println!("  {} - {:.1} RPS, {:.1}% success, {:.2}ms avg latency - {}", 
                 result.test_name, result.requests_per_second, 
                 result.success_rate, result.average_latency_ms, performance_rating);
    }
    
    // Overall assessment
    let avg_rps: f64 = all_results.iter().map(|r| r.requests_per_second).sum::<f64>() / all_results.len() as f64;
    let avg_success: f64 = all_results.iter().map(|r| r.success_rate).sum::<f64>() / all_results.len() as f64;
    let avg_latency: f64 = all_results.iter().map(|r| r.average_latency_ms).sum::<f64>() / all_results.len() as f64;
    
    println!("\n🎯 Overall Performance:");
    println!("  Average RPS: {:.1}", avg_rps);
    println!("  Average Success Rate: {:.1}%", avg_success);
    println!("  Average Latency: {:.2}ms", avg_latency);
    
    if avg_rps > 500.0 && avg_success > 95.0 && avg_latency < 50.0 {
        println!("  🎉 Performance Rating: EXCELLENT - Ready for production!");
    } else if avg_rps > 200.0 && avg_success > 90.0 && avg_latency < 100.0 {
        println!("  ✅ Performance Rating: GOOD - Suitable for most applications");
    } else {
        println!("  ⚠️  Performance Rating: NEEDS IMPROVEMENT");
    }
} 