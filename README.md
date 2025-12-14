# TGP - The Grid Platform

**Author:** vibeswithkk  
**Version:** 0.1.0 (Foundation Phase)  
**License:** Apache 2.0

---

## Overview

**TGP (The Grid Platform)** is a sovereign, open-source AI infrastructure platform that reduces Total Cost of Ownership (TCO) by up to 70% through intelligent economic scheduling and hybrid cloud orchestration.

### Core Components

- **Economic Scheduler** (Rust) - Intelligent job placement with cost optimization
- **Cost Engine** (Rust) - Real-time cost calculation implementing Formula 4.1
- **Optimizer** (Rust) - Placement optimization algorithms
- **API Server** (Go) - REST API for job submission and cluster management

## Quick Start

### Prerequisites

- Rust 1.75+ (`rustup`)
- Go 1.21+
- Make

### Build

```bash
# Build all components
make build

# Run tests
make test

# Start API server
make run-api
```

### API Endpoints

```
GET  /health                    - Health check
POST /api/v1/jobs/submit        - Submit ML job
GET  /api/v1/jobs/:id/status    - Get job status
GET  /api/v1/jobs/:id/cost      - Get cost breakdown
GET  /api/v1/cluster/status     - Cluster status
GET  /api/v1/cluster/nodes      - List nodes
```

## Project Structure

```
TDP/
├── core/
│   ├── scheduler/      # Economic Scheduler engine
│   ├── cost-engine/    # Cost calculation (C_comp, C_data, C_idle)
│   └── optimizer/      # Optimization algorithms
├── api/                # Go REST API server
├── docs/               # Documentation
└── tests/              # Integration tests
```

## Development

```bash
# Format code
make fmt

# Run linters
make lint

# Run benchmarks
make bench

# Generate coverage
make coverage
```

## Architecture

TGP implements a hybrid Rust/Go architecture:

- **Rust Core**: Performance-critical components (scheduler, cost engine)
- **Go API**: High-concurrency HTTP/gRPC server
- **Zero-cost infrastructure**: Designed to run on VPS or on-premise

### Economic Scheduler Formula

```
C_total(J, t) = Σ [C_comp(j,t) + C_data(j,t) + C_idle(t)]

Where:
- C_comp: Compute cost (instance pricing × duration × utilization)
- C_data: Data transfer cost (egress/ingress)
- C_idle: Idle resource opportunity cost
```

## Roadmap

- [x] Phase 1: Project foundation
- [ ] Phase 2: Economic Scheduler core implementation
- [ ] Phase 3: VPS cluster integration
- [ ] Phase 4: AI Mesh Network
- [ ] Phase 5: Model Genetics Engine
- [ ] Phase 6: Community launch

## Contributing

This project is in active development. Contributions welcome!

## License

Apache License 2.0 - See LICENSE file for details