fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../proto/scheduler.proto";
    let proto_dir = "../proto";
    
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(&[proto_file], &[proto_dir])?;
    
    println!("cargo:rerun-if-changed={}", proto_file);
    Ok(())
}
