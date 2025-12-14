//! TGP Worker Agent - Enterprise Grade
//!
//! Responsibilities:
//! - Register with scheduler via gRPC
//! - Report resource availability periodically
//! - Execute assigned jobs in Docker containers
//! - Maintain connection health
//!
//! Design Principles:
//! - Reliability: Automatic reconnection, error handling
//! - Security: TLS support (future), input validation
//! - Maintainability: Clear separation of concerns, documented
//! - Performance: Efficient resource monitoring, minimal overhead
//! - Testability: Modular design, mockable components

mod executor;

use anyhow::{Context, Result};
use std::fs;
use std::time::Duration;
use tonic::transport::Channel;
use tracing::{error, info, warn};

// Include generated gRPC client code
pub mod proto {
    tonic::include_proto!("tgp.scheduler.v1");
}

use proto::{
    scheduler_service_client::SchedulerServiceClient,
    RegisterNodeRequest, ResourceReport,
};

/// Worker configuration
#[derive(Debug, Clone)]
struct WorkerConfig {
    node_id: String,
    scheduler_url: String,
    report_interval_secs: u64,
    reconnect_delay_secs: u64,
    max_retries: u32,
}

impl WorkerConfig {
    fn from_env() -> Self {
        Self {
            node_id: std::env::var("TGP_NODE_ID")
                .unwrap_or_else(|_| hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "worker-unknown".to_string())),
            scheduler_url: std::env::var("TGP_SCHEDULER_URL")
                .unwrap_or_else(|_| "http://YOUR_SCHEDULER_IP:50051".to_string()),
            report_interval_secs: std::env::var("TGP_REPORT_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            reconnect_delay_secs: std::env::var("TGP_RECONNECT_DELAY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            max_retries: std::env::var("TGP_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
        }
    }
}

/// Resource monitoring with error handling
struct ResourceMonitor;

impl ResourceMonitor {
    /// Get hostname with fallback
    fn get_hostname() -> Result<String> {
        fs::read_to_string("/etc/hostname")
            .context("Failed to read hostname")
            .map(|s| s.trim().to_string())
    }

    /// Get CPU count from /proc/cpuinfo
    fn get_cpu_info() -> Result<(u32, u32)> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")
            .context("Failed to read /proc/cpuinfo")?;
        
        let cpu_count = cpuinfo
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count() as u32;

        if cpu_count == 0 {
            anyhow::bail!("No CPUs detected");
        }

        // For now, assume all CPUs available
        // TODO: Check system load
        Ok((cpu_count, cpu_count))
    }

    /// Get memory info from /proc/meminfo
    fn get_memory_info() -> Result<(f64, f64)> {
        let meminfo = fs::read_to_string("/proc/meminfo")
            .context("Failed to read /proc/meminfo")?;
        
        let mut total_kb = None;
        let mut available_kb = None;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            } else if line.starts_with("MemAvailable:") {
                available_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            }

            if total_kb.is_some() && available_kb.is_some() {
                break;
            }
        }

        let total = total_kb.context("MemTotal not found")? as f64 / 1024.0 / 1024.0;
        let available = available_kb.context("MemAvailable not found")? as f64 / 1024.0 / 1024.0;

        Ok((total, available))
    }

    /// Get disk info using df command
    fn get_disk_info() -> Result<(f64, f64)> {
        let output = std::process::Command::new("df")
            .arg("-BG")
            .arg("/")
            .output()
            .context("Failed to execute df command")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let line = stdout
            .lines()
            .nth(1)
            .context("df output too short")?;

        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 4 {
            anyhow::bail!("Invalid df output format");
        }

        let total = parts[1]
            .trim_end_matches('G')
            .parse::<f64>()
            .context("Failed to parse total disk")?;
        
        let available = parts[3]
            .trim_end_matches('G')
            .parse::<f64>()
            .context("Failed to parse available disk")?;

        Ok((total, available))
    }
}

/// TGP Worker Agent
struct WorkerAgent {
    config: WorkerConfig,
    client: Option<SchedulerServiceClient<Channel>>,
}

impl WorkerAgent {
    fn new(config: WorkerConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    /// Connect to scheduler with retry logic
    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to scheduler at {}", self.config.scheduler_url);

        for attempt in 1..=self.config.max_retries {
            match SchedulerServiceClient::connect(self.config.scheduler_url.clone()).await {
                Ok(client) => {
                    info!("Connected to scheduler successfully");
                    self.client = Some(client);
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Connection attempt {}/{} failed: {}",
                        attempt, self.config.max_retries, e
                    );
                    
                    if attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_secs(self.config.reconnect_delay_secs)).await;
                    }
                }
            }
        }

        anyhow::bail!("Failed to connect after {} attempts", self.config.max_retries)
    }

    /// Register node with scheduler
    async fn register(&mut self) -> Result<()> {
        let client = self.client.as_mut()
            .context("Not connected to scheduler")?;

        let hostname = ResourceMonitor::get_hostname()?;
        let (cpu_cores, _) = ResourceMonitor::get_cpu_info()?;
        let (total_memory, _) = ResourceMonitor::get_memory_info()?;

        let request = tonic::Request::new(RegisterNodeRequest {
            node_id: self.config.node_id.clone(),
            hostname,
            cpu_cores,
            total_memory_gb: total_memory,
            gpu_count: 0, // TODO: GPU detection
            location: "vps-2".to_string(), // TODO: Make configurable
            cost_per_hour: 0.1, // TODO: Make configurable
        });

        info!("Registering node: {}", self.config.node_id);

        let response = client
            .register_node(request)
            .await
            .context("Failed to register node")?;

        let reply = response.into_inner();
        
        if reply.success {
            info!("Registration successful: {}", reply.message);
            info!("Assigned to cluster: {}", reply.cluster_id);
        } else {
            error!("Registration failed: {}", reply.message);
            anyhow::bail!("Registration rejected by scheduler");
        }

        Ok(())
    }

    /// Report resources to scheduler
    async fn report_resources(&mut self) -> Result<()> {
        let client = self.client.as_mut()
            .context("Not connected to scheduler")?;

        let (_, available_cpu) = ResourceMonitor::get_cpu_info()
            .unwrap_or((0, 0));
        let (_, available_memory) = ResourceMonitor::get_memory_info()
            .unwrap_or((0.0, 0.0));
        let (_, available_disk) = ResourceMonitor::get_disk_info()
            .unwrap_or((0.0, 0.0));

        let request = tonic::Request::new(ResourceReport {
            node_id: self.config.node_id.clone(),
            available_cpu,
            available_memory_gb: available_memory,
            available_disk_gb: available_disk,
            available_gpu: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        });

        info!(
            "Reporting resources: CPU={}, RAM={:.1}GB, Disk={:.1}GB",
            available_cpu, available_memory, available_disk
        );

        client
            .report_resources(request)
            .await
            .context("Failed to report resources")?;

        Ok(())
    }

    /// Main worker loop with error recovery
    async fn run(&mut self) -> Result<()> {
        info!("TGP Worker Agent starting");
        info!("Node ID: {}", self.config.node_id);
        info!("Scheduler: {}", self.config.scheduler_url);

        // Connect and register
        self.connect().await?;
        self.register().await?;

        // Main loop
        let mut report_interval = tokio::time::interval(
            Duration::from_secs(self.config.report_interval_secs)
        );

        loop {
            report_interval.tick().await;

            // Report resources with error handling
            if let Err(e) = self.report_resources().await {
                error!("Failed to report resources: {}", e);
                
                // Try to reconnect
                warn!("Attempting to reconnect...");
                if let Err(reconnect_err) = self.connect().await {
                    error!("Reconnection failed: {}", reconnect_err);
                    continue;
                }
                
                // Re-register after reconnection
                if let Err(register_err) = self.register().await {
                    error!("Re-registration failed: {}", register_err);
                }
            }
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

    info!("TGP Worker Agent v0.1.0");

    // Load configuration
    let config = WorkerConfig::from_env();

    // Create and run worker
    let mut worker = WorkerAgent::new(config);
    
    match worker.run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Worker failed: {}", e);
            std::process::exit(1);
        }
    }
}
