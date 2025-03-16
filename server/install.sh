#!/bin/bash
set -e  # Exit on error

# Variables
WG_INTERFACE="wg0"
WG_CONFIG="/etc/wireguard/${WG_INTERFACE}.conf"
ENGARDE_URL="https://engarde.linuxzogno.org/builds/master/linux/amd64/engarde-server"
ENGARDE_BINARY="/usr/local/bin/engarde-server"
ENGARDE_CONFIG="/etc/engarde.yaml"
SYSTEMD_SERVICE="/etc/systemd/system/engarde-server.service"
WG_PORT=56882  # Adjusted WireGuard port for non-conflict
ENGARDE_LISTEN="0.0.0.0:59501"
ENGARDE_WEB_PORT="9001"

# Install required packages
echo "Installing WireGuard..."
sudo apt update
sudo apt install -y wireguard iproute2 curl

# Generate WireGuard keys
echo "Generating WireGuard keys..."
WG_PRIVATE_KEY=$(wg genkey)
WG_PUBLIC_KEY=$(echo "$WG_PRIVATE_KEY" | wg pubkey)

# Create WireGuard configuration with a /30 subnet
echo "Setting up WireGuard server..."
sudo bash -c "cat > $WG_CONFIG" <<EOF
[Interface]
Address = 10.88.7.101/30
ListenPort = $WG_PORT
PrivateKey = $WG_PRIVATE_KEY

EOF

# Start and enable WireGuard
sudo systemctl enable --now wg-quick@$WG_INTERFACE

# Download EnGarde server
echo "Downloading EnGarde server..."
sudo curl -L -o "$ENGARDE_BINARY" "$ENGARDE_URL"
sudo chmod +x "$ENGARDE_BINARY"

# Prompt user for the management password
echo -n "Enter password for EnGarde web interface: "
read -s ENGARDE_PASSWORD
echo

# Create EnGarde configuration file
echo "Creating EnGarde configuration file..."
sudo bash -c "cat > $ENGARDE_CONFIG" <<EOF
server:
  description: "Ridgeline Engarde"
  listenAddr: "$ENGARDE_LISTEN"
  dstAddr: "127.0.0.1:$WG_PORT"
  clientTimeout: 30
  writeTimeout: 10
  webManager:
    listenAddr: "0.0.0.0:$ENGARDE_WEB_PORT"
    username: "engarde"
    password: "$ENGARDE_PASSWORD"
EOF

# Create systemd service for EnGarde
echo "Creating systemd service for EnGarde..."
sudo bash -c "cat > $SYSTEMD_SERVICE" <<EOF
[Unit]
Description=Engarde Server
After=network.target

[Service]
ExecStart=$ENGARDE_BINARY $ENGARDE_CONFIG
Restart=always
User=nobody
Group=nogroup
WorkingDirectory=/usr/local/bin
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd, enable and start EnGarde service
sudo systemctl daemon-reload
sudo systemctl enable --now engarde

# Output the WireGuard public key
echo "WireGuard server setup complete!"
echo "Public Key: $WG_PUBLIC_KEY"
