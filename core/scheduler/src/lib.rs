//! TGP Economic Scheduler
//! 
//! Core scheduling engine that optimizes job placement based on cost, performance, and SLA constraints.

pub mod grpc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// The Economic Scheduler - core component of TGP
#[derive(Clone)]
pub struct EconomicScheduler {
    cost_calculator: CostCalculator,
    optimizer: Optimizer,
    available_nodes: HashMap<String, NodeInfo>,
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
            available_nodes: HashMap::new(),
        }
    }

    /// Register a new node in the cluster
    pub fn register_node(&mut self, node: NodeInfo) {
        tracing::info!("Registering node: {} at {}", node.id, node.location);
        self.available_nodes.insert(node.id.clone(), node);
    }

    /// Schedule a job to the optimal node
    /// 
    /// This implements the core Economic Scheduler algorithm:
    /// - Calculate C_total for each possible placement
    /// - Validate SLA constraints
    /// - Select placement that minimizes cost while satisfying SLA
    pub async fn schedule(&self, job: JobSpec) -> Result<Placement> {
        tracing::info!("Scheduling job: {}", job.id);

        if self.available_nodes.is_empty() {
            anyhow::bail!("No nodes available in cluster");
        }

        let mut best_placement: Option<Placement> = None;
        let mut min_cost = f64::MAX;

        // Evaluate each node for placement
        for node in self.available_nodes.values() {
            // Check resource availability
            if !self.check_resource_fit(&job.resources, node) {
                tracing::debug!("Node {} insufficient resources", node.id);
                continue;
            }

            // Calculate total cost for this placement
            let estimated_duration = 1.0; // TODO: estimate based on job type
            let data_size = 0.0; // TODO: get from job spec
            
            let cost = self.cost_calculator.total_cost(
                node.cost_per_hour,
                estimated_duration,
                1.0, // 100% utilization during job
                data_size,
                0.0, // VPS-to-VPS transfer is free
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

            // Track best placement (minimum cost)
            if cost.total_usd < min_cost {
                min_cost = cost.total_usd;
                best_placement = Some(Placement {
                    job_id: job.id.clone(),
                    node_id: node.id.clone(),
                    estimated_cost: cost,
                    estimated_latency_ms: estimated_latency,
                });
                tracing::info!(
                    "New best placement: {} on node {} (cost: ${:.4})",
                    job.id, node.id, min_cost
                );
            }
        }

        best_placement.ok_or_else(|| anyhow::anyhow!("No suitable node found for job {}", job.id))
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

    /// Get cluster status
    pub fn cluster_status(&self) -> Vec<NodeInfo> {
        self.available_nodes.values().cloned().collect()
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
        assert_eq!(scheduler.available_nodes.len(), 0);
    }

    #[test]
    fn test_node_registration() {
        let mut scheduler = EconomicScheduler::new();
        let node = NodeInfo {
            id: "test-node-1".to_string(),
            available_cpu: 8,
            available_memory_gb: 32,
            available_gpu: 1,
            location: "vps-1".to_string(),
            cost_per_hour: 0.5,
        };

        scheduler.register_node(node.clone());
        assert_eq!(scheduler.available_nodes.len(), 1);
        assert!(scheduler.available_nodes.contains_key("test-node-1"));
    }
}
