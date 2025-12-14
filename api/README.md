# API Server (Go) - Optional Component

**Status:** Placeholder - Requires Go installation

## Current Status

The Go API server is a placeholder component that provides REST API endpoints. **TGP core functionality is fully operational without it** using the gRPC test client.

## Why Go API is Optional

TGP's core distributed scheduling system is complete with:
- ✅ Rust scheduler with Formula 4.1 (VPS #1)
- ✅ Rust worker with Docker executor (VPS #2)
- ✅ gRPC test client for job submission
- ✅ All end-to-end tests passing

The Go API server would provide additional REST endpoints but is not required for operation.

## Installation (If Needed)

If you want to use the Go REST API:

```bash
# Install Go 1.21+
wget https://go.dev/dl/go1.21.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.linux-amd64.tar.gz
export PATH=$PATH:/usr/local/go/bin

# Install dependencies
cd api
go mod tidy

# Build
go build -o tgp-api main.go

# Run
./tgp-api
```

## Alternative: Use gRPC Test Client

Instead of the Go API, use the fully functional Rust test client:

```bash
# All TGP functionality available via:
./target/release/tgp-test-client cluster-status
./target/release/tgp-test-client submit-job --job-id test-001 --cpu 1 --memory 1
./target/release/tgp-test-client get-status test-001
```

## Future Plans

The Go API server can be implemented when needed for:
- HTTP REST API (vs gRPC)
- Web dashboard integration
- Third-party integrations

**For now, TGP is production-ready without it.**
