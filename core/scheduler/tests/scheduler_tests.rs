#[cfg(test)]
mod scheduler_tests {
    use tgp_scheduler::{EconomicScheduler, JobSpec, JobType, NodeInfo, ResourceRequirements, SlaConstraints};

    #[tokio::test]
    async fn test_schedule_selects_cheapest_node() {
        let mut scheduler = EconomicScheduler::new();

        // Register two nodes with different costs
        scheduler.register_node(NodeInfo {
            id: "cheap-node".to_string(),
            available_cpu: 4,
            available_memory_gb: 8,
            available_gpu: 0,
            location: "vps-1".to_string(),
            cost_per_hour: 0.25, // Cheaper
        });

        scheduler.register_node(NodeInfo {
            id: "expensive-node".to_string(),
            available_cpu: 8,
            available_memory_gb: 16,
            available_gpu: 1,
            location: "vps-2".to_string(),
            cost_per_hour: 1.0, // More expensive
        });

        let job = JobSpec {
            id: "test-job-1".to_string(),
            job_type: JobType::Inference,
            resources: ResourceRequirements {
                cpu_cores: 2,
                memory_gb: 4,
                gpu_count: 0,
                disk_gb: 10,
            },
            sla: SlaConstraints {
                max_latency_ms: 1000,
                max_budget_usd: None,
                deadline: None,
            },
        };

        let placement = scheduler.schedule(job).await.unwrap();
        
        // Should select cheaper node
        assert_eq!(placement.node_id, "cheap-node");
        assert!(placement.estimated_cost.total_usd < 1.0);
    }

    #[tokio::test]
    async fn test_schedule_respects_sla_budget() {
        let mut scheduler = EconomicScheduler::new();

        scheduler.register_node(NodeInfo {
            id: "cheap-node".to_string(),
            available_cpu: 2,
            available_memory_gb: 4,
            available_gpu: 0,
            location: "vps-1".to_string(),
            cost_per_hour: 0.1,
        });

        scheduler.register_node(NodeInfo {
            id: "expensive-node".to_string(),
            available_cpu: 8,
            available_memory_gb: 16,
            available_gpu: 1,
            location: "vps-2".to_string(),
            cost_per_hour: 10.0, // Very expensive
        });

        let job = JobSpec {
            id: "budget-constrained-job".to_string(),
            job_type: JobType::Training,
            resources: ResourceRequirements {
                cpu_cores: 2,
                memory_gb: 4,
                gpu_count: 0,
                disk_gb: 10,
            },
            sla: SlaConstraints {
                max_latency_ms: 5000,
                max_budget_usd: Some(0.5), // Budget constraint
                deadline: None,
            },
        };

        let placement = scheduler.schedule(job).await.unwrap();
        
        // Should select cheap node due to budget constraint
        assert_eq!(placement.node_id, "cheap-node");
        assert!(placement.estimated_cost.total_usd <= 0.5);
    }

    #[tokio::test]
    async fn test_schedule_fails_insufficient_resources() {
        let mut scheduler = EconomicScheduler::new();

        scheduler.register_node(NodeInfo {
            id: "small-node".to_string(),
            available_cpu: 1,
            available_memory_gb: 1,
            available_gpu: 0,
            location: "vps-1".to_string(),
            cost_per_hour: 0.1,
        });

        let job = JobSpec {
            id: "large-job".to_string(),
            job_type: JobType::Training,
            resources: ResourceRequirements {
                cpu_cores: 8, // Too many cores
                memory_gb: 32, // Too much memory
                gpu_count: 0,
                disk_gb: 100,
            },
            sla: SlaConstraints {
                max_latency_ms: 5000,
                max_budget_usd: None,
                deadline: None,
            },
        };

        let result = scheduler.schedule(job).await;
        
        // Should fail - no suitable node
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No suitable node"));
    }

    #[tokio::test]
    async fn test_schedule_no_nodes_available() {
        let scheduler = EconomicScheduler::new(); // No nodes registered

        let job = JobSpec {
            id: "orphan-job".to_string(),
            job_type: JobType::Inference,
            resources: ResourceRequirements {
                cpu_cores: 1,
                memory_gb: 1,
                gpu_count: 0,
                disk_gb: 10,
            },
            sla: SlaConstraints {
                max_latency_ms: 1000,
                max_budget_usd: None,
                deadline: None,
            },
        };

        let result = scheduler.schedule(job).await;
        
        // Should fail - no nodes in cluster
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No nodes available"));
    }
}
