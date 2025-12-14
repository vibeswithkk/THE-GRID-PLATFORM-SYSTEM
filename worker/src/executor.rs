//! Worker Job Executor - Docker-based job execution
//!
//! Executes jobs in isolated Docker containers with resource limits
//! according to TGP blueprint specifications

#![allow(dead_code, unused_imports)]

use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions, WaitContainerOptions,
};
use bollard::models::HostConfig;
use bollard::Docker;
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Job execution request from scheduler
#[derive(Debug, Clone)]
pub struct JobExecution {
    pub job_id: String,
    pub job_type: String,
    pub container_image: String,
    pub cpu_limit: u32,
    pub memory_limit_mb: u64,
    pub command: Option<Vec<String>>,
    pub env: HashMap<String, String>,
}

/// Job executor using Docker containers
pub struct JobExecutor {
    docker: Docker,
}

impl JobExecutor {
    /// Create new job executor
    pub fn new() -> Result<Self> {
        // Connect to Docker daemon on local Unix socket
        let docker = Docker::connect_with_socket_defaults()
            .context("Failed to connect to Docker daemon")?;

        Ok(Self { docker })
    }

    /// Execute a job in a Docker container
    pub async fn execute_job(&self, job: JobExecution) -> Result<JobResult> {
        info!("Executing job {} with image {}", job.job_id, job.container_image);

        // Pull image if not exists
        self.pull_image(&job.container_image).await?;

        // Create container with resource limits
        let container_id = self.create_container(&job).await?;

        // Start container
        info!("Starting container: {}", container_id);
        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start container")?;

        // Wait for container to complete
        let exit_code = self.wait_for_completion(&container_id).await?;

        // Get container logs
        let logs = self.get_logs(&container_id).await?;

        // Clean up container
        self.cleanup_container(&container_id).await?;

        let result = JobResult {
            job_id: job.job_id.clone(),
            success: exit_code == 0,
            exit_code,
            logs,
            error: if exit_code != 0 {
                Some(format!("Container exited with code {}", exit_code))
            } else {
                None
            },
        };

        if result.success {
            info!("Job {} completed successfully", job.job_id);
        } else {
            error!("Job {} failed with exit code {}", job.job_id, exit_code);
        }

        Ok(result)
    }

    /// Pull Docker image
    async fn pull_image(&self, image: &str) -> Result<()> {
        use bollard::image::CreateImageOptions;
        use futures_util::stream::StreamExt;

        info!("Pulling image: {}", image);

        let options = Some(CreateImageOptions {
            from_image: image,
            ..Default::default()
        });

        let mut stream = self.docker.create_image(options, None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        if status.contains("Download") || status.contains("Pull") {
                            info!("{}", status);
                        }
                    }
                }
                Err(e) => {
                    warn!("Image pull warning: {}", e);
                }
            }
        }

        info!("Image {} ready", image);
        Ok(())
    }

    /// Create container with resource limits
    async fn create_container(&self, job: &JobExecution) -> Result<String> {
        // Set resource limits according to TGP blueprint
        let host_config = HostConfig {
            cpu_quota: Some((job.cpu_limit as i64) * 100_000), // CPU quota in microseconds
            memory: Some((job.memory_limit_mb * 1024 * 1024) as i64), // Memory in bytes
            memory_swap: Some((job.memory_limit_mb * 1024 * 1024) as i64), // No swap
            network_mode: Some("bridge".to_string()),
            auto_remove: Some(false), // We'll remove manually after getting logs
            ..Default::default()
        };

        let config = Config {
            image: Some(job.container_image.clone()),
            cmd: job.command.clone(),
            env: Some(
                job.env
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect(),
            ),
            host_config: Some(host_config),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: format!("tgp-job-{}", job.job_id),
            platform: None,
        };

        let response = self
            .docker
            .create_container(Some(options), config)
            .await
            .context("Failed to create container")?;

        info!("Container created: {}", response.id);
        Ok(response.id)
    }

    /// Wait for container to complete
    async fn wait_for_completion(&self, container_id: &str) -> Result<i64> {
        use futures_util::stream::StreamExt;

        info!("Waiting for container {} to complete", container_id);

        let options = Some(WaitContainerOptions {
            condition: "not-running",
        });

        let mut stream = self.docker.wait_container(container_id, options);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    let code = response.status_code;
                    info!("Container exited with code: {}", code);
                    return Ok(code);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Error waiting for container: {}", e));
                }
            }
        }

        Ok(0)
    }

    /// Get container logs
    async fn get_logs(&self, container_id: &str) -> Result<String> {
        use futures_util::stream::StreamExt;

        let options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: "all".to_string(),
            ..Default::default()
        });

        let mut stream = self.docker.logs(container_id, options);
        let mut logs = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    logs.push_str(&output.to_string());
                }
                Err(e) => {
                    warn!("Error reading logs: {}", e);
                }
            }
        }

        Ok(logs)
    }

    /// Clean up container after execution
    async fn cleanup_container(&self, container_id: &str) -> Result<()> {
        info!("Cleaning up container: {}", container_id);

        // Stop if still running
        let _ = self
            .docker
            .stop_container(container_id, None::<StopContainerOptions>)
            .await;

        // Remove container
        let options = Some(RemoveContainerOptions {
            force: true,
            v: true, // Remove volumes
            ..Default::default()
        });

        self.docker
            .remove_container(container_id, options)
            .await
            .context("Failed to remove container")?;

        info!("Container removed: {}", container_id);
        Ok(())
    }
}

/// Job execution result
#[derive(Debug, Clone)]
pub struct JobResult {
    pub job_id: String,
    pub success: bool,
    pub exit_code: i64,
    pub logs: String,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker daemon
    async fn test_execute_simple_job() {
        let executor = JobExecutor::new().unwrap();

        let job = JobExecution {
            job_id: "test-job-1".to_string(),
            job_type: "test".to_string(),
            container_image: "alpine:latest".to_string(),
            cpu_limit: 1,
            memory_limit_mb: 128,
            command: Some(vec!["echo".to_string(), "Hello from TGP!".to_string()]),
            env: HashMap::new(),
        };

        let result = executor.execute_job(job).await.unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!(result.logs.contains("Hello from TGP"));
    }
}
