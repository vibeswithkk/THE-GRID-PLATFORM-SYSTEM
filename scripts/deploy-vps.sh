#!/bin/bash
# TGP Deployment Script for VPS
# SECURITY: Use environment variables for credentials

set -e

# Load credentials from environment or .env file
if [ -f ".env.deployment" ]; then
    source .env.deployment
fi

# Validate required environment variables
if [ -z "$VPS1_IP" ] || [ -z "$VPS1_USER" ] || [ -z "$VPS1_PASS" ]; then
    echo "ERROR: Missing required environment variables!"
    echo "Please set: VPS1_IP, VPS1_USER, VPS1_PASS"
    echo ""
    echo "Example: Create .env.deployment file with:"
    echo "  export VPS1_IP=\"your-vps-ip\""
    echo "  export VPS1_USER=\"your-username\""
    echo "  export VPS1_PASS=\"your-password\""
    exit 1
fi

echo "=== TGP Deployment to VPS #1 ==="
echo "Target: $VPS1_IP"
echo "Building deployment package..."

# Create deployment tarball
tar -czf /tmp/tgp-deploy.tar.gz \
    --exclude='target' \
    --exclude='.git' \
    --exclude='*.tar.gz' \
    --exclude='.env*' \
    Cargo.toml Cargo.lock \
    core/ \
    proto/ \
    worker/ \
    Dockerfile.scheduler \
    docker-compose.yml \
    Makefile \
    README.md

echo "Package created: $(du -h /tmp/tgp-deploy.tar.gz | cut -f1)"

echo "Transferring to VPS #1..."
sshpass -p "$VPS1_PASS" scp -o StrictHostKeyChecking=no \
    /tmp/tgp-deploy.tar.gz \
    $VPS1_USER@$VPS1_IP:/root/

echo "Deploying on VPS..."
sshpass -p "$VPS1_PASS" ssh -o StrictHostKeyChecking=no \
    $VPS1_USER@$VPS1_IP bash <<'ENDSSH'
cd /root
rm -rf TDP
mkdir -p TDP
cd TDP
tar -xzf ../tgp-deploy.tar.gz
rm ../tgp-deploy.tar.gz

echo "Building Docker image..."
docker-compose build

echo "Starting scheduler..."
docker-compose up -d

echo "Checking status..."
sleep 3
docker-compose ps
docker-compose logs scheduler | tail -20

echo ""
echo "=== Deployment Complete ==="
ENDSSH

echo ""
echo "[SUCCESS] TGP Scheduler deployed to VPS #1!"
echo "Access: http://$VPS1_IP:50051"
