#!/bin/bash
# TGP Deployment Script for VPS
set -e

VPS1_IP="202.155.157.122"
VPS1_USER="root"
VPS1_PASS="@@wahyu123OK"

echo "=== TGP Deployment to VPS #1 ==="
echo "Building deployment package..."

# Create deployment tarball
tar -czf /tmp/tgp-deploy.tar.gz \
    --exclude='target' \
    --exclude='.git' \
    --exclude='*.tar.gz' \
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
echo "✅ TGP Scheduler deployed to VPS #1!"
echo "Access: http://$VPS1_IP:50051"
