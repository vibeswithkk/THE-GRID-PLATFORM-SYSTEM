//! gRPC server implementation for TGP Scheduler
//! 
//! Implements the SchedulerService defined in scheduler.proto

use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

use crate::EconomicScheduler;

// Include generated proto code
pub mod proto {
    tonic::include_proto!("tgp.scheduler.v1");
}

use proto::{
    scheduler_service_server::{SchedulerService, SchedulerServiceServer},
    *,
};

#[tonic::async_trait]
impl SchedulerService for EconomicScheduler {
    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> Result<Response<RegisterNodeResponse>, Status> {
        let req = request.into_inner();
        info!("Registering node: {} ({})", req.node_id, req.hostname);

        // Use actual scheduler to register node
        let node = crate::NodeInfo {
            id: req.node_id.clone(),
            available_cpu: req.cpu_cores,
            available_memory_gb: (req.total_memory_gb as u32),
            available_gpu: req.gpu_count,
            location: req.location.clone(),
            cost_per_hour: req.cost_per_hour,
        };

        match self.register_node(node) {
            Ok(_) => {
                info!("Node {} registered in scheduler", req.node_id);
                let response = RegisterNodeResponse {
                    success: true,
                    message: format!("Node {} registered successfully", req.node_id),
                    cluster_id: "tgp-cluster-1".to_string(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                Err(Status::internal(format!("Failed to register node: {}", e)))
            }
        }
    }

    async fn report_resources(
        &self,
        request: Request<ResourceReport>,
    ) -> Result<Response<ResourceAck>, Status> {
        let report = request.into_inner();
        
        info!(
            "Resource report from {}: CPU={}, RAM={:.1}GB, Disk={:.1}GB",
            report.node_id,
            report.available_cpu,
            report.available_memory_gb,
            report.available_disk_gb
        );

        // TODO: Update node resources (requires update_node_resources method)
        // For now, just acknowledge receipt

        Ok(Response::new(ResourceAck { received: true }))
    }

    async fn submit_job(
        &self,
        request: Request<JobSubmitRequest>,
    ) -> Result<Response<JobSubmitResponse>, Status> {
        let job_req = request.into_inner();
        
        info!("Job submission: {} (type: {:?})", job_req.job_id, job_req.job_type);

        // Convert proto types to scheduler types
        let job_spec = crate::JobSpec {
            id: job_req.job_id.clone(),
            job_type: match job_req.job_type {
                1 => crate::JobType::Training,
                2 => crate::JobType::Inference,
                3 => crate::JobType::DataProcessing,
                _ => crate::JobType::Inference,
            },
            resources: crate::ResourceRequirements {
                cpu_cores: job_req.resources.as_ref()
                    .map(|r| r.cpu_cores)
                    .unwrap_or(1),
                memory_gb: job_req.resources.as_ref()
                    .map(|r| r.memory_gb)
                    .unwrap_or(1),
                gpu_count: job_req.resources.as_ref()
                    .map(|r| r.gpu_count)
                    .unwrap_or(0),
                disk_gb: job_req.resources.as_ref()
                    .map(|r| r.disk_gb)
                    .unwrap_or(10),
            },
            sla: crate::SlaConstraints {
                max_latency_ms: job_req.sla.as_ref()
                    .map(|s| s.max_latency_ms)
                    .unwrap_or(1000),
                max_budget_usd: job_req.sla.as_ref()
                    .and_then(|s| s.max_budget_usd),
                deadline: job_req.sla.as_ref()
                    .and_then(|s| s.deadline),
            },
        };

        // Use actual scheduler with Formula 4.1
        match self.schedule(job_spec).await {
            Ok(placement) => {
                info!(
                    "Job {} scheduled to {} with Formula 4.1 TCO ${:.4}",
                    placement.job_id,
                    placement.node_id,
                    placement.estimated_cost.total_usd
                );

                let response = JobSubmitResponse {
                    success: true,
                    job_id: placement.job_id,
                    assigned_node: placement.node_id,
                    cost_estimate: Some(CostEstimate {
                        compute_cost_usd: placement.estimated_cost.compute_usd,
                        data_transfer_usd: placement.estimated_cost.data_transfer_usd,
                        idle_opportunity_usd: placement.estimated_cost.idle_opportunity_usd,
                        total_cost_usd: placement.estimated_cost.total_usd,
                        estimated_latency_ms: placement.estimated_latency_ms,
                    }),
                    message: format!(
                        "Job scheduled using Formula 4.1 - TCO: ${:.4}",
                        placement.estimated_cost.total_usd
                    ),
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                Err(Status::internal(format!("Scheduling failed: {}", e)))
            }
        }
    }

    async fn get_job_status(
        &self,
        request: Request<JobStatusRequest>,
    ) -> Result<Response<JobStatusResponse>, Status> {
        let req = request.into_inner();
        
        // Query actual job state
        match self.get_job_state(&req.job_id) {
            Some(state) => {
                let proto_status = match state.status {
                    crate::JobStatus::Pending => JobStatus::Pending.into(),
                    crate::JobStatus::Scheduled => JobStatus::Scheduled.into(),
                    crate::JobStatus::Running => JobStatus::Running.into(),
                    crate::JobStatus::Completed => JobStatus::Completed.into(),
                    crate::JobStatus::Failed => JobStatus::Failed.into(),
                };

                let final_cost = state.estimated_cost.map(|cost| CostEstimate {
                    compute_cost_usd: cost.compute_usd,
                    data_transfer_usd: cost.data_transfer_usd,
                    idle_opportunity_usd: cost.idle_opportunity_usd,
                    total_cost_usd: cost.total_usd,
                    estimated_latency_ms: 0, // TODO: track actual latency
                });

                let response = JobStatusResponse {
                    job_id: req.job_id,
                    status: proto_status,
                    assigned_node: state.assigned_node.unwrap_or_default(),
                    final_cost,
                };

                Ok(Response::new(response))
            }
            None => {
                Err(Status::not_found(format!("Job {} not found", req.job_id)))
            }
        }
    }

    async fn get_cluster_status(
        &self,
        _request: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusResponse>, Status> {
        info!("Cluster status requested");

        // Get actual cluster status
        let nodes_info = self.cluster_status();
        
        let proto_nodes: Vec<NodeInfo> = nodes_info.iter().map(|node| NodeInfo {
            node_id: node.id.clone(),
            hostname: node.id.clone(), // TODO: store actual hostname
            available_cpu: node.available_cpu,
            available_memory_gb: node.available_memory_gb as f64,
            location: node.location.clone(),
            is_active: true,
        }).collect();
        
        let response = ClusterStatusResponse {
            total_nodes: nodes_info.len() as u32,
            active_nodes: nodes_info.len() as u32,
            total_jobs: 0, // TODO: track total jobs
            running_jobs: 0, // TODO: track running jobs
            nodes: proto_nodes,
        };

        Ok(Response::new(response))
    }
}

/// Start gRPC server
pub async fn start_grpc_server(
    scheduler: EconomicScheduler,
    addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(SchedulerServiceServer::new(scheduler))
        .serve(addr)
        .await?;

    Ok(())
}
