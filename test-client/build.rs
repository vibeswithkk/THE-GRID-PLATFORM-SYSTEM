fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile proto for test client
    tonic_build::compile_protos("../proto/scheduler.proto")?;
    Ok(())
}
