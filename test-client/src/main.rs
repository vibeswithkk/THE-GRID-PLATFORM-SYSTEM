//! TGP Test Client - CLI tool for testing scheduler
//!
//! Submit jobs, query status, and test cost optimization

use anyhow::Result;
use clap::{Parser, Subcommand};
use tonic::Request;
use tracing::info;

// Include generated proto code
pub mod proto {
    tonic::include_proto!("tgp.scheduler.v1");
}

use proto::{
    scheduler_service_client::SchedulerServiceClient, JobSubmitRequest, JobType,
    ResourceRequirements, SlaConstraints, JobStatusRequest, ClusterStatusRequest,
};

#[derive(Parser)]
#[command(name = "tgp-test-client")]
#[command(about = "TGP Test Client - Submit jobs and test scheduler", long_about = None)]
struct Cli {
    /// Scheduler address
    #[arg(short, long, default_value = "http://202.155.157.122:50051")]
    scheduler: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a test job
    SubmitJob {
        /// Job ID
        #[arg(short, long)]
        job_id: String,

        /// Container image
        #[arg(short, long, default_value = "alpine:latest")]
        image: String,

        /// CPU cores required
        #[arg(long, default_value = "1")]
        cpu: u32,

        /// Memory in GB
        #[arg(long, default_value = "1")]
        memory: u32,

        /// Max budget in USD
        #[arg(long)]
        budget: Option<f64>,

        /// Max latency in ms
        #[arg(long, default_value = "1000")]
        latency: u64,
    },

    /// Get job status
    GetStatus {
        /// Job ID
        job_id: String,
    },

    /// Get cluster status
    ClusterStatus,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let cli = Cli::parse();

    // Connect to scheduler
    info!("Connecting to scheduler at {}", cli.scheduler);
    let mut client = SchedulerServiceClient::connect(cli.scheduler.clone()).await?;
    info!("Connected successfully!");

    match cli.command {
        Commands::SubmitJob {
            job_id,
            image,
            cpu,
            memory,
            budget,
            latency,
        } => {
            submit_job(&mut client, job_id, image, cpu, memory, budget, latency).await?;
        }
        Commands::GetStatus { job_id } => {
            get_job_status(&mut client, job_id).await?;
        }
        Commands::ClusterStatus => {
            get_cluster_status(&mut client).await?;
        }
    }

    Ok(())
}

async fn submit_job(
    client: &mut SchedulerServiceClient<tonic::transport::Channel>,
    job_id: String,
    _image: String,
    cpu: u32,
    memory: u32,
    budget: Option<f64>,
    latency: u64,
) -> Result<()> {
    info!("Submitting job: {}", job_id);
    info!("Resources: {} CPU, {}GB RAM", cpu, memory);
    if let Some(b) = budget {
        info!("Budget: ${:.2}", b);
    }
    info!("Max latency: {}ms", latency);

    let request = Request::new(JobSubmitRequest {
        job_id: job_id.clone(),
        job_type: JobType::Inference.into(),
        resources: Some(ResourceRequirements {
            cpu_cores: cpu,
            memory_gb: memory,
            gpu_count: 0,
            disk_gb: 10,
        }),
        sla: Some(SlaConstraints {
            max_latency_ms: latency,
            max_budget_usd: budget,
            deadline: None,
        }),
        job_data: vec![],
    });

    let response = client.submit_job(request).await?;
    let job = response.into_inner();

    if job.success {
        println!("\n✅ Job Submitted Successfully!");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Job ID:        {}", job.job_id);
        println!("Assigned Node: {}", job.assigned_node);
        
        if let Some(cost) = job.cost_estimate {
            println!("\nCost Estimate (Formula 4.1):");
            println!("  C_comp (Compute):     ${:.6}", cost.compute_cost_usd);
            println!("  C_data (Transfer):    ${:.6}", cost.data_transfer_usd);
            println!("  C_idle (Opportunity): ${:.6}", cost.idle_opportunity_usd);
            println!("  ─────────────────────────────");
            println!("  C_total (TCO):        ${:.6}", cost.total_cost_usd);
            println!("  Estimated Latency:    {}ms", cost.estimated_latency_ms);
        }
        
        println!("\nMessage: {}", job.message);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    } else {
        println!("\n❌ Job Submission Failed!");
        println!("Message: {}", job.message);
    }

    Ok(())
}

async fn get_job_status(
    client: &mut SchedulerServiceClient<tonic::transport::Channel>,
    job_id: String,
) -> Result<()> {
    info!("Querying status for job: {}", job_id);

    let request = Request::new(JobStatusRequest { job_id: job_id.clone() });
    let response = client.get_job_status(request).await?;
    let status = response.into_inner();

    println!("\n📊 Job Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Job ID:        {}", status.job_id);
    println!("Status:        {:?}", status.status);
    println!("Assigned Node: {}", status.assigned_node);
    
    if let Some(cost) = status.final_cost {
        println!("\nFinal Cost:");
        println!("  C_total: ${:.6}", cost.total_cost_usd);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

async fn get_cluster_status(
    client: &mut SchedulerServiceClient<tonic::transport::Channel>,
) -> Result<()> {
    info!("Querying cluster status");

    let request = Request::new(ClusterStatusRequest {});
    let response = client.get_cluster_status(request).await?;
    let cluster = response.into_inner();

    println!("\n🌐 Cluster Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total Nodes:   {}", cluster.total_nodes);
    println!("Active Nodes:  {}", cluster.active_nodes);
    println!("Total Jobs:    {}", cluster.total_jobs);
    println!("Running Jobs:  {}", cluster.running_jobs);
    
    if !cluster.nodes.is_empty() {
        println!("\nRegistered Nodes:");
        for node in cluster.nodes {
            println!("\n  Node: {}", node.node_id);
            println!("    Hostname:   {}", node.hostname);
            println!("    CPU:        {}", node.available_cpu);
            println!("    Memory:     {:.1}GB", node.available_memory_gb);
            println!("    Location:   {}", node.location);
            println!("    Active:     {}", node.is_active);
        }
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
