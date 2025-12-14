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

        // TODO: Actually register the node in scheduler
        // For now, just acknowledge
        
        let response = RegisterNodeResponse {
            success: true,
            message: format!("Node {} registered successfully", req.node_id),
            cluster_id: "tgp-cluster-1".to_string(),
        };

        Ok(Response::new(response))
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

        // TODO: Update node resources in scheduler state

        Ok(Response::new(ResourceAck { received: true }))
    }

    async fn submit_job(
        &self,
        request: Request<JobSubmitRequest>,
    ) -> Result<Response<JobSubmitResponse>, Status> {
        let job_req = request.into_inner();
        
        info!("Job submission: {} (type: {:?})", job_req.job_id, job_req.job_type);

        // TODO: Actually schedule the job using EconomicScheduler::schedule()
        
        let response = JobSubmitResponse {
            success: true,
            job_id: job_req.job_id.clone(),
            assigned_node: "vps-2".to_string(),
            cost_estimate: Some(CostEstimate {
                compute_cost_usd: 0.5,
                data_transfer_usd: 0.0,
                idle_opportunity_usd: 0.0,
                total_cost_usd: 0.5,
                estimated_latency_ms: 100,
            }),
            message: "Job scheduled successfully".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn get_job_status(
        &self,
        request: Request<JobStatusRequest>,
    ) -> Result<Response<JobStatusResponse>, Status> {
        let req = request.into_inner();
        
        // TODO: Query actual job status
        
        let response = JobStatusResponse {
            job_id: req.job_id,
            status: JobStatus::Running.into(),
            assigned_node: "vps-2".to_string(),
            final_cost: None,
        };

        Ok(Response::new(response))
    }

    async fn get_cluster_status(
        &self,
        _request: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusResponse>, Status> {
        info!("Cluster status requested");

        // TODO: Get actual cluster status from scheduler
        
        let response = ClusterStatusResponse {
            total_nodes: 2,
            active_nodes: 2,
            total_jobs: 0,
            running_jobs: 0,
            nodes: vec![],
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
