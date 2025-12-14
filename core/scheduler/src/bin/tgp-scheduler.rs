//! TGP Scheduler Binary
//! 
//! Main entry point for the TGP Economic Scheduler service

use tgp_scheduler::EconomicScheduler;
use tracing_subscriber;

#[tokio::main]
async fn main() {
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
    tracing::info!("TODO: Start gRPC server on port 50051");
    
    // For now, keep running
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
    
    tracing::info!("Scheduler shutting down");
}
