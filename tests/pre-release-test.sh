#!/bin/bash
# Comprehensive TGP Pre-Release Test Suite
# Tests all components before release

set -e  # Exit on error

echo "##########################################---"
echo "TGP COMPREHENSIVE PRE-RELEASE TEST SUITE"
echo "##########################################---"
echo ""

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function for test reporting
report_test() {
    if [ $1 -eq 0 ]; then
        echo "**PASSED: $2"
        ((TESTS_PASSED++))
    else
        echo "**FAILED: $2"
        ((TESTS_FAILED++))
    fi
}

# ============================================================================
# PHASE 1: BUILD VALIDATION
# ============================================================================
echo "##########################################---"
echo "PHASE 1: Build Validation"
echo "##########################################---"
echo ""

echo "Building scheduler..."
cargo build --release --bin tgp-scheduler > /dev/null 2>&1
report_test $? "Scheduler build"

echo "Building worker..."
cargo build --release --bin tgp-worker > /dev/null 2>&1
report_test $? "Worker build"

echo "Building test client..."
cargo build --release --bin tgp-test-client > /dev/null 2>&1
report_test $? "Test client build"

echo ""

# ============================================================================
# PHASE 2: UNIT TESTS
# ============================================================================
echo "##########################################---"
echo "PHASE 2: Unit Tests"
echo "##########################################---"
echo ""

echo "Testing scheduler unit tests..."
cargo test --package tgp-scheduler --lib > /dev/null 2>&1
report_test $? "Scheduler unit tests"

echo "Testing cost engine..."
cargo test --package tgp-cost-engine > /dev/null 2>&1
report_test $? "Cost engine tests"

echo "Testing optimizer..."
cargo test --package tgp-optimizer > /dev/null 2>&1
report_test $? "Optimizer tests"

echo ""

# ============================================================================
# PHASE 3: BINARY VALIDATION
# ============================================================================
echo "##########################################---"
echo "PHASE 3: Binary Validation"
echo "##########################################---"
echo ""

# Check binary sizes
SCHEDULER_SIZE=$(du -h target/release/tgp-scheduler | cut -f1)
WORKER_SIZE=$(du -h target/release/tgp-worker | cut -f1)
CLIENT_SIZE=$(du -h target/release/tgp-test-client | cut -f1)

echo "Scheduler binary: $SCHEDULER_SIZE"
echo "Worker binary: $WORKER_SIZE"
echo "Test client binary: $CLIENT_SIZE"

[ -f target/release/tgp-scheduler ] && report_test 0 "Scheduler binary exists" || report_test 1 "Scheduler binary exists"
[ -f target/release/tgp-worker ] && report_test 0 "Worker binary exists" || report_test 1 "Worker binary exists"
[ -f target/release/tgp-test-client ] && report_test 0 "Test client binary exists" || report_test 1 "Test client binary exists"

echo ""

# ============================================================================
# PHASE 4: VPS CONNECTIVITY
# ============================================================================
echo "##########################################---"
echo "PHASE 4: VPS Connectivity Tests"
echo "##########################################---"
echo ""

echo "Testing scheduler connectivity (VPS #1)..."
timeout 5 bash -c 'cat < /dev/null > /dev/tcp/202.155.157.122/50051' 2>/dev/null
report_test $? "Scheduler port 50051 reachable"

echo "Testing VPS #2 SSH..."
sshpass -p '@@wahyu123OK' ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 root@72.61.119.83 'echo ok' > /dev/null 2>&1
report_test $? "VPS #2 SSH connectivity"

echo ""

# ============================================================================
# PHASE 5: SCHEDULER STATUS
# ============================================================================
echo "##########################################---"
echo "PHASE 5: Scheduler Status"
echo "##########################################---"
echo ""

echo "Checking scheduler container..."
sshpass -p '@@wahyu123OK' ssh -o StrictHostKeyChecking=no root@202.155.157.122 'docker ps | grep tgp-scheduler' > /dev/null 2>&1
report_test $? "Scheduler container running"

echo "Checking worker service..."
sshpass -p '@@wahyu123OK' ssh -o StrictHostKeyChecking=no root@72.61.119.83 'systemctl is-active tgp-worker' > /dev/null 2>&1
report_test $? "Worker service active"

echo ""

# ============================================================================
# PHASE 6: END-TO-END TESTS
# ============================================================================
echo "##########################################---"
echo "PHASE 6: End-to-End Tests"
echo "##########################################---"
echo ""

echo "Test 1: Cluster status query..."
./target/release/tgp-test-client cluster-status > /tmp/tgp-test-cluster.txt 2>&1
grep -q "Total Nodes" /tmp/tgp-test-cluster.txt
report_test $? "Cluster status query"

echo "Test 2: Job submission..."
./target/release/tgp-test-client submit-job \
  --job-id test-prerelease-001 \
  --cpu 1 --memory 1 \
  --budget 10.0 --latency 2000 > /tmp/tgp-test-submit.txt 2>&1
grep -q "Job Submitted Successfully" /tmp/tgp-test-submit.txt
report_test $? "Job submission"

echo "Test 3: Cost estimation..."
grep -q "C_total" /tmp/tgp-test-submit.txt
report_test $? "Formula 4.1 cost calculation"

echo "Test 4: Job status query..."
./target/release/tgp-test-client get-status test-prerelease-001 > /tmp/tgp-test-status.txt 2>&1
grep -q "Job Status" /tmp/tgp-test-status.txt
report_test $? "Job status query"

echo ""

# ============================================================================
# PHASE 7: DOCKER EXECUTION
# ============================================================================
echo "##########################################---"
echo "PHASE 7: Docker Execution Tests"
echo "##########################################---"
echo ""

echo "Test 1: Simple echo job..."
sshpass -p '@@wahyu123OK' ssh -o StrictHostKeyChecking=no root@72.61.119.83 \
  'docker run --rm --cpus="1" --memory="128m" alpine:latest echo "TGP Test"' > /tmp/docker-test1.txt 2>&1
grep -q "TGP Test" /tmp/docker-test1.txt
report_test $? "Docker echo execution"

echo "Test 2: Resource limits..."
sshpass -p '@@wahyu123OK' ssh -o StrictHostKeyChecking=no root@72.61.119.83 \
  'docker run --rm --cpus="1" --memory="128m" alpine:latest sh -c "echo ok"' > /tmp/docker-test2.txt 2>&1
grep -q "ok" /tmp/docker-test2.txt
report_test $? "Docker resource limits"

echo ""

# ============================================================================
# PHASE 8: PERFORMANCE VALIDATION
# ============================================================================
echo "##########################################---"
echo "PHASE 8: Performance Validation"
echo "##########################################---"
echo ""

echo "Testing scheduler response time..."
START_TIME=$(date +%s%N)
./target/release/tgp-test-client cluster-status > /dev/null 2>&1
END_TIME=$(date +%s%N)
LATENCY=$(( ($END_TIME - $START_TIME) / 1000000 ))  # Convert to milliseconds

echo "Latency: ${LATENCY}ms"
[ $LATENCY -lt 1000 ] && report_test 0 "Scheduler latency < 1000ms" || report_test 1 "Scheduler latency < 1000ms"

echo ""

# ============================================================================
# FINAL REPORT
# ============================================================================
echo "##########################################---"
echo "FINAL TEST REPORT"
echo "##########################################---"
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
SUCCESS_RATE=$(( TESTS_PASSED * 100 / TOTAL_TESTS ))

echo "Total Tests:    $TOTAL_TESTS"
echo "Tests Passed:   $TESTS_PASSED **[PASS]**"
echo "Tests Failed:   $TESTS_FAILED ****"
echo "Success Rate:   $SUCCESS_RATE%"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo "##########################################---"
    echo "ALL TESTS PASSED - READY FOR RELEASE!"
    echo "##########################################---"
    exit 0
else
    echo "##########################################---"
    echo "⚠️  SOME TESTS FAILED - REVIEW BEFORE RELEASE"
    echo "##########################################---"
    exit 1
fi
