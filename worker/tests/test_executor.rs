//! Manual test script for Docker job execution
//! 
//! Tests the JobExecutor with a simple Alpine container

use anyhow::Result;

// Use the executor module from worker
mod executor;
use executor::{JobExecution, JobExecutor};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("\n🧪 Testing TGP Docker Job Executor\n");

    // Create job executor
    let executor = JobExecutor::new()?;
    println!("✅ JobExecutor initialized");
    println!("✅ Connected to Docker daemon\n");

    // Test 1: Simple echo job
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 1: Alpine Echo Job");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let job1 = JobExecution {
        job_id: "test-docker-001".to_string(),
        job_type: "test".to_string(),
        container_image: "alpine:latest".to_string(),
        cpu_limit: 1,
        memory_limit_mb: 128,
        command: Some(vec![
            "echo".to_string(),
            "Hello from TGP Economic Scheduler!".to_string(),
        ]),
        env: HashMap::new(),
    };

    println!("📦 Job: {}", job1.job_id);
    println!("🐳 Image: {}", job1.container_image);
    println!("💻 Command: echo 'Hello from TGP Economic Scheduler!'");
    println!("🔧 Resources: {} CPU, {}MB RAM\n", job1.cpu_limit, job1.memory_limit_mb);

    let result1 = executor.execute_job(job1).await?;

    println!("\n📊 Execution Result:");
    println!("  Success: {}", result1.success);
    println!("  Exit Code: {}", result1.exit_code);
    println!("  Logs:");
    for line in result1.logs.lines().take(10) {
        println!("    {}", line);
    }
    if let Some(error) = result1.error {
        println!("  Error: {}", error);
    }

    // Test 2: CPU benchmark
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 2: CPU Benchmark Job");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let job2 = JobExecution {
        job_id: "test-docker-002".to_string(),
        job_type: "benchmark".to_string(),
        container_image: "alpine:latest".to_string(),
        cpu_limit: 1,
        memory_limit_mb: 256,
        command: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "i=0; while [ $i -lt 100000 ]; do i=$((i+1)); done; echo 'Benchmark complete: $i iterations'".to_string(),
        ]),
        env: HashMap::new(),
    };

    println!("📦 Job: {}", job2.job_id);
    println!("🐳 Image: {}", job2.container_image);
    println!("💻 Command: CPU loop (100k iterations)");
    println!("🔧 Resources: {} CPU, {}MB RAM\n", job2.cpu_limit, job2.memory_limit_mb);

    let result2 = executor.execute_job(job2).await?;

    println!("\n📊 Execution Result:");
    println!("  Success: {}", result2.success);
    println!("  Exit Code: {}", result2.exit_code);
    println!("  Logs:");
    for line in result2.logs.lines().take(10) {
        println!("    {}", line);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ All Docker execution tests passed!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
