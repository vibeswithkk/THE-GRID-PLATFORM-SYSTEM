#!/bin/bash
# Deploy worker agent to VPS #2
# SECURITY: Use environment variables for credentials

set -e

# Load credentials from environment or .env file
if [ -f ".env.deployment" ]; then
    source .env.deployment
fi

# Validate required environment variables
if [ -z "$VPS2_IP" ] || [ -z "$VPS2_USER" ] || [ -z "$VPS2_PASS" ]; then
    echo "ERROR: Missing required environment variables!"
    echo "Please set: VPS2_IP, VPS2_USER, VPS2_PASS"
    echo ""
    echo "Example: Create .env.deployment file with:"
    echo "  export VPS2_IP=\"your-vps-ip\""
    echo "  export VPS2_USER=\"your-username\""
    echo "  export VPS2_PASS=\"your-password\""
    exit 1
fi

echo "=== Deploying TGP Worker to VPS #2 ==="
echo "Target: $VPS2_IP"

# Build worker locally (already done)
echo "Binary: target/release/tgp-worker"
du -h target/release/tgp-worker | cut -f1

echo "Transferring to VPS #2..."
sshpass -p "$VPS2_PASS" scp -o StrictHostKeyChecking=no \
    target/release/tgp-worker \
    $VPS2_USER@$VPS2_IP:/usr/local/bin/

echo "Setting up systemd service..."
sshpass -p "$VPS2_PASS" ssh -o StrictHostKeyChecking=no \
    $VPS2_USER@$VPS2_IP bash <<'ENDSSH'
    
# Make executable
chmod +x /usr/local/bin/tgp-worker

# Note: Update scheduler URL in systemd service as needed
SCHEDULER_URL="${TGP_SCHEDULER_URL:-http://YOUR_SCHEDULER_IP:50051}"

# Create systemd service
cat > /etc/systemd/system/tgp-worker.service <<EOF
[Unit]
Description=TGP Worker Agent
After=network.target

[Service]
Type=simple
User=root
Environment="TGP_NODE_ID=vps-2"
Environment="TGP_SCHEDULER_URL=$SCHEDULER_URL"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/tgp-worker
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start worker
systemctl daemon-reload
systemctl enable tgp-worker
systemctl restart tgp-worker

# Check status
sleep 2
systemctl status tgp-worker --no-pager

echo ""
echo "=== Worker logs ==="
journalctl -u tgp-worker -n 20 --no-pager

ENDSSH

echo ""
echo "[SUCCESS] TGP Worker deployed to VPS #2!"
