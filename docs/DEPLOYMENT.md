# TGP Deployment Guide - VPS Setup

**Last Updated:** 2025-12-14  
**VPS Nodes:** 2

---

## Quick Deploy

### VPS #1 (zenith1) - Control Plane

```bash
# SSH to VPS #1
ssh root@202.155.157.122

# Clone repository
git clone git@github.com:vibeswithkk/TDP.git
cd TDP

# Build and deploy scheduler
docker-compose up -d

# Verify scheduler running
docker ps
curl localhost:50051 # Should connect to scheduler gRPC
```

### VPS #2 (srv1133629) - Worker Node

```bash
# SSH to VPS #2
ssh root@72.61.119.83

# Install TGP worker agent (coming in next phase)
# For now, this VPS is ready for workload execution
```

---

## Architecture

```
VPS #1 (zenith1)              VPS #2 (srv1133629)
202.155.157.122               72.61.119.83
├─ TGP Scheduler              ├─ Worker Agent
│  └─ Port 50051 (gRPC)       │  └─ Reports resources
└─ Cost Calculator            └─ Executes jobs
```

---

## What's Deployed

****Economic Scheduler** - Cost-based job placement  
****Cost Calculator** - Formula 4.1 (C_total)  
****SLA Validation** - Budget, latency, deadline constraints  
****Resource Checking** - CPU, RAM, GPU availability  

---

## Test Commands

```bash
# Test scheduler locally
cd /home/viuren/VELTRUMNISCEND/TDP
cargo test --workspace

# Expected: 10/10 tests passing
```

---

## Next Steps

1. Create worker agent binary
2. Deploy to VPS #2
3. Test end-to-end job submission
4. Benchmark cost optimization
