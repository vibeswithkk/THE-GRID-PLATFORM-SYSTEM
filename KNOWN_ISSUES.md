# Known Issues

## Go API Server Dependencies

**Issue:** IDE shows errors in `api/main.go` for missing Go packages.

**Cause:** Go is not installed on the development system.

**Impact:** None - Go API is an optional component. TGP core functionality is fully operational.

**Status:** ****NOT BLOCKING** - System works without Go API

### Working Components

All critical TGP components are functional:
- **Scheduler (Rust) - Running on VPS #1
- **Worker (Rust) - Running on VPS #2  
- **Test Client (Rust) - Job submission working
- **Docker Execution - Validated
- **Formula 4.1 TCO - Verified
- **End-to-end tests - All passing

### Workaround

Use the gRPC test client instead of REST API:

```bash
# Submit jobs
./target/release/tgp-test-client submit-job --job-id my-job --cpu 1 --memory 1

# Check status
./target/release/tgp-test-client get-status my-job

# Cluster info
./target/release/tgp-test-client cluster-status
```

### Resolution (Optional)

If REST API is needed in future:

```bash
# Install Go
wget https://go.dev/dl/go1.21.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.linux-amd64.tar.gz

# Setup environment
export PATH=$PATH:/usr/local/go/bin

# Install dependencies
cd api && go mod tidy

# Build
go build -o tgp-api main.go
```

**Recommendation:** Keep using gRPC test client - it provides all needed functionality.
