#!/bin/bash
# Deploy worker agent to VPS #2
set -e

VPS2_IP="72.61.119.83"
VPS2_USER="root"
VPS2_PASS="@@wahyu123OK"

echo "=== Deploying TGP Worker to VPS #2 ==="

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

# Create systemd service
cat > /etc/systemd/system/tgp-worker.service <<EOF
[Unit]
Description=TGP Worker Agent
After=network.target

[Service]
Type=simple
User=root
Environment="TGP_NODE_ID=vps-2"
Environment="TGP_SCHEDULER_URL=http://202.155.157.122:50051"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/tgp-worker
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start worker
systemd daemon-reload
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
echo "✅ TGP Worker deployed to VPS #2!"
