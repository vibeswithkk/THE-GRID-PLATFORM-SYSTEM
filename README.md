<p align="center">
  <h1 align="center">TGP - The Grid Platform</h1>
  <p align="center">
    <strong>Open-Source Economic Job Scheduler for AI Infrastructure</strong>
  </p>
  <p align="center">
    <em>Reduce your AI infrastructure costs by 30% with Formula 4.1 TCO optimization</em>
  </p>
</p>

<p align="center">
  <a href="https://opensource.org/licenses/Apache-2.0"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License"></a>
  <a href="#"><img src="https://img.shields.io/badge/build-passing-brightgreen" alt="Build"></a>
  <a href="#"><img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version"></a>
  <a href="#"><img src="https://img.shields.io/badge/rust-1.75%2B-orange" alt="Rust"></a>
  <a href="https://tgp-dashboard.vercel.app"><img src="https://img.shields.io/badge/demo-live-success" alt="Demo"></a>
</p>

<p align="center">
  <a href="https://tgp-dashboard.vercel.app">Live Demo</a> •
  <a href="docs/WHITEPAPER.md">Whitepaper</a> •
  <a href="docs/blog/">Blog</a> •
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

## Why TGP?

Traditional job schedulers optimize for **resource utilization**. TGP optimizes for **cost**.

```
Your current scheduler:  "Put this job wherever there's space"
TGP:                     "Put this job where it costs the least while meeting SLA"
```

### The Hidden Cost Problem

Most teams don't realize they're overspending on AI infrastructure:
- **Data transfer costs** buried in cloud bills
- **Idle GPUs** burning money 24/7
- **Wrong instance types** from fear of job failures

TGP solves this with **Formula 4.1**:

```
C_total = C_comp + C_data + C_idle
```

**Real results:** 15-30% cost reduction with zero SLA violations.

---

## Website & Live Demo

**Try TGP without installing:** [https://tgp-dashboard.vercel.app](https://tgp-dashboard.vercel.app)

The live demo includes:
- Real-time cluster status monitoring
- Job submission interface
- Cost breakdown visualization (Formula 4.1)
- Full dashboard experience

---

## Quick Start

### Prerequisites
- Rust 1.75+ 
- Docker
- Protocol Buffers compiler

### Installation

```bash
# Clone
git clone https://github.com/vibeswithkk/TGP.git
cd TGP

# Build
cargo build --release

# Verify
./target/release/tgp-scheduler --version
```

### Submit Your First Job

```bash
# Check cluster status
./target/release/tgp-test-client cluster-status

# Submit a job
./target/release/tgp-test-client submit-job \
  --job-id my-first-job \
  --cpu 2 --memory 4 \
  --budget 5.0 --latency 1000
```

---

## Architecture

```
┌──────────────┐         ┌─────────────────────────────────────┐
│   Client     │──gRPC──▶│           TGP Scheduler             │
│  (CLI/API)   │         │                                     │
└──────────────┘         │  ┌─────────────┐  ┌──────────────┐  │
                         │  │ Cost Engine │  │  Optimizer   │  │
                         │  │ Formula 4.1 │  │  (Greedy)    │  │
                         │  └─────────────┘  └──────────────┘  │
                         └────────────────┬────────────────────┘
                                          │
                         ┌────────────────┼────────────────┐
                         ▼                ▼                ▼
                   ┌──────────┐    ┌──────────┐    ┌──────────┐
                   │ Worker 1 │    │ Worker 2 │    │ Worker N │
                   │ (Docker) │    │ (Docker) │    │ (Docker) │
                   └──────────┘    └──────────┘    └──────────┘
```

### Components

| Component | Language | Purpose |
|-----------|----------|---------|
| `tgp-scheduler` | Rust | Core scheduler with Formula 4.1 |
| `tgp-cost-engine` | Rust | TCO calculation engine |
| `tgp-optimizer` | Rust | Placement optimization |
| `tgp-worker` | Rust | Job execution agent |
| `dashboard` | Next.js | Web UI for monitoring |

---

## Formula 4.1: The Economics

TGP's scheduling decisions are driven by our Total Cost of Operation model:

| Cost Component | Formula | Description |
|----------------|---------|-------------|
| **C_comp** | `price × duration × utilization` | Compute cost |
| **C_data** | `data_size × transfer_price` | Data transfer cost |
| **C_idle** | `idle_hours × opportunity_cost` | Idle resource cost |

**Example:**
```
Job: train-resnet-50
Location Option A (Cloud): C_total = $2.50 + $0.90 + $0.00 = $3.40
Location Option B (On-Prem): C_total = $0.00 + $0.00 + $0.50 = $0.50

TGP Decision: Run on-premise, save $2.90 (85%)
```

[Read the full whitepaper →](docs/WHITEPAPER.md)

---

## Performance

| Metric | Value |
|--------|-------|
| Scheduling Latency | ~30ms |
| Cost Savings | 15-30% |
| SLA Violations | 0% |
| Languages | Rust, Go |

---

## Live Demo

Try TGP without installing anything:

**[→ tgp-dashboard.vercel.app](https://tgp-dashboard.vercel.app)**

The demo shows:
- Real-time cluster status
- Job submission and tracking
- Cost breakdown visualization (Formula 4.1)
- Resource utilization metrics

---

## Documentation

| Document | Description |
|----------|-------------|
| [Whitepaper](docs/WHITEPAPER.md) | Technical deep-dive on Formula 4.1 |
| [Deployment Guide](docs/DEPLOYMENT.md) | VPS setup instructions |
| [Blog](docs/blog/) | Technical articles |
| [Contributing](CONTRIBUTING.md) | How to contribute |

---

## Roadmap

### Completed
- [x] Economic Scheduler with Formula 4.1
- [x] gRPC distributed communication
- [x] Docker-based job execution
- [x] Web dashboard

### Coming Soon
- [ ] Kubernetes integration
- [ ] Spot instance intelligence
- [ ] Multi-cloud support (AWS, GCP, Azure)
- [ ] Carbon-aware scheduling
- [ ] ML-based cost prediction

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Setup
git clone https://github.com/vibeswithkk/TGP.git
cd TGP
cargo build

# Test
cargo test

# Lint
cargo clippy
```

---

## Community

- **GitHub Issues**: Bug reports & feature requests
- **GitHub Discussions**: Questions & ideas

---

## License

Apache License 2.0 - see [LICENSE](LICENSE)

---

<p align="center">
  <strong>TGP</strong> - Making AI infrastructure economically efficient
  <br>
  <a href="https://tgp-dashboard.vercel.app">Demo</a> •
  <a href="docs/WHITEPAPER.md">Whitepaper</a> •
  <a href="https://github.com/vibeswithkk/TGP">GitHub</a>
</p>