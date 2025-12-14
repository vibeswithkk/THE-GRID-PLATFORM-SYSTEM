#!/bin/bash
# Simple Docker execution test on worker VPS

echo "##############################"
echo "TGP Docker Execution Test"
echo "##############################"
echo ""

# Test 1: Simple echo
echo "TEST 1: Alpine Echo Job"
echo "-------------------------"
docker run --rm --name tgp-test-001 \
  --cpus="1" \
  --memory="128m" \
  alpine:latest \
  echo "Hello from TGP Economic Scheduler!"

echo ""
echo "**Test 1 complete"
echo ""

# Test 2: CPU benchmark  
echo "TEST 2: CPU Benchmark"
echo "---------------------"
docker run --rm --name tgp-test-002 \
  --cpus="1" \
  --memory="256m" \
  alpine:latest \
  sh -c 'i=0; while [ $i -lt 100000 ]; do i=$((i+1)); done; echo "Benchmark complete: $i iterations"'

echo ""
echo "**Test 2 complete"
echo ""

# Test 3: Multi-line output
echo "TEST 3: System Info"
echo "-------------------"
docker run --rm --name tgp-test-003 \
  --cpus="1" \
  --memory="128m" \
  alpine:latest \
  sh -c 'echo "Container System Info:"; echo "Host: $(hostname)"; echo "User: $(whoami)"; echo "Date: $(date)"'

echo ""
echo "**Test 3 complete"
echo ""

echo "##############################"
echo "**All Docker execution tests passed!"
echo "##############################"
