//! TGP Economic Scheduler
//! 
//! Core scheduling engine that optimizes job placement based on cost, performance, and SLA constraints.

pub mod grpc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tgp_cost_engine::{CostCalculator, TotalCost};
use tgp_optimizer::Optimizer;

/// Job specification submitted by users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    /// Unique job identifier
    pub id: String,
    /// Job type (training, inference, etc.)
    pub job_type: JobType,
    /// Resource requirements
    pub resources: ResourceRequirements,
    /// SLA constraints
    pub sla: SlaConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    Training,
    Inference,
    DataProcessing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_count: u32,
    pub disk_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaConstraints {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: u64,
    /// Maximum budget in USD
    pub max_budget_usd: Option<f64>,
    /// Deadline timestamp
    pub deadline: Option<i64>,
}

/// Placement decision for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    pub job_id: String,
    pub node_id: String,
    pub estimated_cost: TotalCost,
    pub estimated_latency_ms: u64,
}

/// Job status tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
}

/// Job state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub job_id: String,
    pub status: JobStatus,
    pub assigned_node: Option<String>,
    pub estimated_cost: Option<TotalCost>,
}

/// The Economic Scheduler - core component of TGP (Thread-Safe)
#[derive(Clone)]
pub struct EconomicScheduler {
    cost_calculator: CostCalculator,
    optimizer: Optimizer,
    /// Thread-safe node registry for concurrent gRPC access
    available_nodes: Arc<Mutex<HashMap<String, NodeInfo>>>,
    /// Thread-safe job state tracking
    job_states: Arc<Mutex<HashMap<String, JobState>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub available_cpu: u32,
    pub available_memory_gb: u32,
    pub available_gpu: u32,
    pub location: String,
    pub cost_per_hour: f64,
}

impl EconomicScheduler {
    /// Create a new Economic Scheduler instance
    pub fn new() -> Self {
        Self {
            cost_calculator: CostCalculator::new(),
            optimizer: Optimizer::new(),
            available_nodes: Arc::new(Mutex::new(HashMap::new())),
            job_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new node in the cluster (thread-safe)
    pub fn register_node(&self, node: NodeInfo) -> Result<()> {
        tracing::info!("Registering node: {} at {}", node.id, node.location);
        
        let mut nodes = self.available_nodes.lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Schedule a job to the optimal node (Thread-Safe with Formula 4.1)
    /// 
    /// This implements the core Economic Scheduler algorithm:
    /// - Calculate C_total for each possible placement using Formula 4.1
    /// - Validate SLA constraints
    /// - Select placement that minimizes TCO while satisfying SLA
    pub async fn schedule(&self, job: JobSpec) -> Result<Placement> {
        tracing::info!("Scheduling job: {} (Formula 4.1)", job.id);

        // Create initial job state
        {
            let mut states = self.job_states.lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            
            states.insert(job.id.clone(), JobState {
                job_id: job.id.clone(),
                status: JobStatus::Pending,
                assigned_node: None,
                estimated_cost: None,
            });
        }

        // Get nodes snapshot for scheduling
        let nodes = {
            let nodes_lock = self.available_nodes.lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            nodes_lock.clone()
        };

        if nodes.is_empty() {
            self.update_job_state(job.id.clone(), JobStatus::Failed, None)?;
            anyhow::bail!("No nodes available in cluster");
        }

        let mut best_placement: Option<Placement> = None;
        let mut min_cost = f64::MAX;

        // Evaluate each node for placement
        for node in nodes.values() {
            // Check resource availability
            if !self.check_resource_fit(&job.resources, node) {
                tracing::debug!("Node {} insufficient resources", node.id);
                continue;
            }

            // Calculate total cost for this placement using Formula 4.1
            // C_total = C_comp + C_data + C_idle
            let estimated_duration = 1.0; // TODO: estimate based on job type
            let data_size = 0.0; // TODO: get from job spec
            
            let cost = self.cost_calculator.total_cost(
                node.cost_per_hour,
                estimated_duration,
                1.0, // 100% utilization during job
                data_size,
                0.0, // VPS-to-VPS transfer is free (blueprint assumption)
                0.0, // No idle cost during active job
                0.0,
            );

            // Estimate latency based on node load
            let estimated_latency = self.estimate_latency(node);

            // Check SLA constraints
            if estimated_latency > job.sla.max_latency_ms {
                tracing::debug!("Node {} violates SLA latency requirement", node.id);
                continue;
            }

            if let Some(max_budget) = job.sla.max_budget_usd {
                if cost.total_usd > max_budget {
                    tracing::debug!("Node {} exceeds budget constraint", node.id);
                    continue;
                }
            }

            // Track best placement (minimum cost - Formula 4.1 TCO optimization)
            if cost.total_usd < min_cost {
                min_cost = cost.total_usd;
                best_placement = Some(Placement {
                    job_id: job.id.clone(),
                    node_id: node.id.clone(),
                    estimated_cost: cost.clone(),
                    estimated_latency_ms: estimated_latency,
                });
                tracing::info!(
                    "Formula 4.1: Best placement {} on node {} (TCO: ${:.4})",
                    job.id, node.id, min_cost
                );
            }
        }

        match best_placement {
            Some(placement) => {
                // Update job state to Scheduled
                self.update_job_state(
                    job.id.clone(),
                    JobStatus::Scheduled,
                    Some(placement.node_id.clone())
                )?;
                
                // Store cost estimate
                {
                    let mut states = self.job_states.lock()
                        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
                    if let Some(state) = states.get_mut(&job.id) {
                        state.estimated_cost = Some(placement.estimated_cost.clone());
                    }
                }
                
                tracing::info!("Job {} scheduled to {} with TCO ${:.4}", 
                    job.id, placement.node_id, placement.estimated_cost.total_usd);
                Ok(placement)
            }
            None => {
                self.update_job_state(job.id.clone(), JobStatus::Failed, None)?;
                anyhow::bail!("No suitable node found for job {} (Formula 4.1 constraints)", job.id)
            }
        }
    }

    /// Get node count (thread-safe)
    pub fn node_count(&self) -> usize {
        self.available_nodes.lock()
            .map(|nodes| nodes.len())
            .unwrap_or(0)
    }

    /// Get job state (thread-safe)
    pub fn get_job_state(&self, job_id: &str) -> Option<JobState> {
        self.job_states.lock()
            .ok()
            .and_then(|states| states.get(job_id).cloned())
    }

    /// Update job state (thread-safe)
    pub fn update_job_state(&self, job_id: String, status: JobStatus, assigned_node: Option<String>) -> Result<()> {
        let mut states = self.job_states.lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        if let Some(state) = states.get_mut(&job_id) {
            state.status = status;
            if let Some(node) = assigned_node {
                state.assigned_node = Some(node);
            }
        }
        
        Ok(())
    }

    /// Check if node has sufficient resources for job
    fn check_resource_fit(&self, required: &ResourceRequirements, node: &NodeInfo) -> bool {
        node.available_cpu >= required.cpu_cores
            && node.available_memory_gb >= required.memory_gb
            && node.available_gpu >= required.gpu_count
    }

    /// Estimate job latency based on node characteristics
    fn estimate_latency(&self, node: &NodeInfo) -> u64 {
        // Simple estimation: base latency + resource pressure
        let base_latency = 50; // 50ms base
        
        // Add latency if node is heavily utilized
        let cpu_pressure = if node.available_cpu < 2 { 50 } else { 0 };
        let mem_pressure = if node.available_memory_gb < 2 { 30 } else { 0 };
        
        base_latency + cpu_pressure + mem_pressure
    }

    /// Get cluster status (thread-safe)
    pub fn cluster_status(&self) -> Vec<NodeInfo> {
        self.available_nodes.lock()
            .map(|nodes| nodes.values().cloned().collect())
            .unwrap_or_else(|_| Vec::new())
    }
}

impl Default for EconomicScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = EconomicScheduler::new();
        assert_eq!(scheduler.node_count(), 0);
    }

    #[test]
    fn test_node_registration() {
        let scheduler = EconomicScheduler::new();
        let node = NodeInfo {
            id: "test-node-1".to_string(),
            available_cpu: 8,
            available_memory_gb: 32,
            available_gpu: 1,
            location: "vps-1".to_string(),
            cost_per_hour: 0.5,
        };

        scheduler.register_node(node.clone()).unwrap();
        assert_eq!(scheduler.node_count(), 1);
    }
}
