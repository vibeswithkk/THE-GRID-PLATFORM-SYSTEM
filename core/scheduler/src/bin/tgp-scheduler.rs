//! TGP Scheduler Binary
//! 
//! Main entry point for the TGP Economic Scheduler service

use tgp_scheduler::EconomicScheduler;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting TGP Economic Scheduler v0.1.0");

    // Create scheduler instance
    let scheduler = EconomicScheduler::new();

    tracing::info!("Scheduler initialized");

    // Start gRPC server
    let addr = "0.0.0.0:50051".parse()?;
    tracing::info!("Starting gRPC server on {}", addr);
    
    tgp_scheduler::grpc::start_grpc_server(scheduler, addr).await?;

    Ok(())
}
