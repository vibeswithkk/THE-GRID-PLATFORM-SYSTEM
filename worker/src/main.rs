//! TGP Worker Agent
//!
//! Runs on worker nodes to:
//! - Report resource availability to scheduler
//! - Execute assigned jobs
//! - Send health checks / heartbeats

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct ResourceReport {
    node_id: String,
    hostname: String,
    cpu_cores: u32,
    available_cpu: u32,
    total_memory_gb: f64,
    available_memory_gb: f64,
    total_disk_gb: f64,
    available_disk_gb: f64,
    gpu_count: u32,
}

struct WorkerAgent {
    node_id: String,
    scheduler_url: String,
}

impl WorkerAgent {
    fn new(node_id: String, scheduler_url: String) -> Self {
        Self {
            node_id,
            scheduler_url,
        }
    }

    /// Get current resource availability
    fn get_resources(&self) -> Result<ResourceReport> {
        let hostname = self.get_hostname()?;
        let (cpu_cores, available_cpu) = self.get_cpu_info()?;
        let (total_mem, available_mem) = self.get_memory_info()?;
        let (total_disk, available_disk) = self.get_disk_info()?;

        Ok(ResourceReport {
            node_id: self.node_id.clone(),
            hostname,
            cpu_cores,
            available_cpu,
            total_memory_gb: total_mem,
            available_memory_gb: available_mem,
            total_disk_gb: total_disk,
            available_disk_gb: available_disk,
            gpu_count: 0, // TODO: GPU detection
        })
    }

    fn get_hostname(&self) -> Result<String> {
        let hostname = fs::read_to_string("/etc/hostname")?
            .trim()
            .to_string();
        Ok(hostname)
    }

    fn get_cpu_info(&self) -> Result<(u32, u32)> {
        // Read /proc/cpuinfo for CPU count
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
        let cpu_count = cpuinfo.lines()
            .filter(|line| line.starts_with("processor"))
            .count() as u32;

        // For simplicity, assume all CPUs available (TODO: check load)
        let available = cpu_count;

        Ok((cpu_count, available))
    }

    fn get_memory_info(&self) -> Result<(f64, f64)> {
        // Read /proc/meminfo
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }

        let total_gb = total_kb as f64 / 1024.0 / 1024.0;
        let available_gb = available_kb as f64 / 1024.0 / 1024.0;

        Ok((total_gb, available_gb))
    }

    fn get_disk_info(&self) -> Result<(f64, f64)> {
        // Read df output for root filesystem
        let output = std::process::Command::new("df")
            .arg("-BG")
            .arg("/")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse df output (skip header)
        if let Some(line) = stdout.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts[1].trim_end_matches('G').parse::<f64>().unwrap_or(0.0);
                let available = parts[3].trim_end_matches('G').parse::<f64>().unwrap_or(0.0);
                return Ok((total, available));
            }
        }

        Ok((0.0, 0.0))
    }

    /// Report resources to scheduler (would use gRPC in production)
    async fn report_to_scheduler(&self) -> Result<()> {
        let resources = self.get_resources()?;
        
        info!(
            "Resource Report - CPU: {}/{}, RAM: {:.1}/{:.1}GB, Disk: {:.0}/{:.0}GB",
            resources.available_cpu,
            resources.cpu_cores,
            resources.available_memory_gb,
            resources.total_memory_gb,
            resources.available_disk_gb,
            resources.total_disk_gb
        );

        // TODO: Send to scheduler via gRPC
        // For now, just log
        info!("Would send to scheduler: {}", self.scheduler_url);

        Ok(())
    }

    /// Main worker loop
    async fn run(&self) -> Result<()> {
        info!("TGP Worker Agent starting on node: {}", self.node_id);
        
        loop {
            // Report resources every 10 seconds
            match self.report_to_scheduler().await {
                Ok(_) => {},
                Err(e) => warn!("Failed to report resources: {}", e),
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Get configuration from environment
    let node_id = std::env::var("TGP_NODE_ID")
        .unwrap_or_else(|_| "worker-1".to_string());
    
    let scheduler_url = std::env::var("TGP_SCHEDULER_URL")
        .unwrap_or_else(|_| "http://202.155.157.122:50051".to_string());

    info!("Starting TGP Worker Agent v0.1.0");
    info!("Node ID: {}", node_id);
    info!("Scheduler: {}", scheduler_url);

    let worker = WorkerAgent::new(node_id, scheduler_url);
    worker.run().await?;

    Ok(())
}
