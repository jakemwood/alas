#!/bin/bash
set -e  # Exit on error

# Variables
WG_INTERFACE="wg0"
WG_CONFIG="/etc/wireguard/${WG_INTERFACE}.conf"
WIREGUARD_IP="10.88.7.101"

ENGARDE_URL="https://engarde.linuxzogno.org/builds/master/linux/amd64/engarde-server"
ENGARDE_BINARY="/usr/local/bin/engarde-server"
ENGARDE_CONFIG="/etc/engarde.yaml"
ENGARDE_SYSTEMD_SERVICE="/etc/systemd/system/engarde-server.service"

WG_PORT=56882  # Adjusted WireGuard port for non-conflict
ENGARDE_LISTEN="0.0.0.0:59501"
ENGARDE_WEB_PORT="9001"
ICECAST_CONFIG="/etc/icecast2/icecast.xml"
# Get public IP address
PUBLIC_IP=$(curl -s https://api.ipify.org)


# Generate a random password for Icecast
ICECAST_PASSWORD=$(openssl rand -base64 16)

# Install required packages
echo "Installing WireGuard and Icecast..."
sudo apt update
sudo apt install -y wireguard iproute2 curl icecast2

# Configure Icecast with the random password
echo "Configuring Icecast..."
sudo bash -c "cat > $ICECAST_CONFIG" <<EOF
<icecast>
    <limits>
        <clients>5</clients>
        <sources>2</sources>
        <queue-size>524288</queue-size>
        <client-timeout>30</client-timeout>
        <header-timeout>15</header-timeout>
        <source-timeout>10</source-timeout>
        <burst-size>65535</burst-size>
    </limits>

    <authentication>
        <source-password>$ICECAST_PASSWORD</source-password>
        <relay-password>$ICECAST_PASSWORD</relay-password>
        <admin-user>admin</admin-user>
        <admin-password>$ICECAST_PASSWORD</admin-password>
    </authentication>

    <hostname>10.88.7.101</hostname>

    <listen-socket>
        <port>8000</port>
    </listen-socket>

    <fileserve>1</fileserve>

    <paths>
        <basedir>/usr/share/icecast2</basedir>
        <logdir>/var/log/icecast2</logdir>
        <webroot>/usr/share/icecast2/web</webroot>
        <adminroot>/usr/share/icecast2/admin</adminroot>
        <alias source="/" destination="/status.xsl"/>
    </paths>

    <logging>
        <accesslog>access.log</accesslog>
        <errorlog>error.log</errorlog>
        <loglevel>3</loglevel>
    </logging>

    <security>
        <chroot>0</chroot>
    </security>
</icecast>
EOF

# Restart Icecast to apply new configuration
sudo systemctl restart icecast2
sudo systemctl enable icecast2

# Generate WireGuard keys
echo "Generating WireGuard keys..."
WG_PRIVATE_KEY=$(wg genkey)
WG_PUBLIC_KEY=$(echo "$WG_PRIVATE_KEY" | wg pubkey)

# Create WireGuard configuration with a /30 subnet
echo "Setting up WireGuard server..."
sudo bash -c "cat > $WG_CONFIG" <<EOF
[Interface]
Address = $WIREGUARD_IP/30
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
echo "Creating systemd service for Engarde..."
sudo bash -c "cat > $ENGARDE_SYSTEMD_SERVICE" <<EOF
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
sudo systemctl enable --now engarde-server

# Output the configuration information
echo "Installation complete!"
echo "----------------------------------------"
echo "WireGuard Public Key: $WG_PUBLIC_KEY"
echo "Icecast Password: $ICECAST_PASSWORD"
echo "----------------------------------------"
echo "Icecast is running on port 8000"
echo "EnGarde is running on port $ENGARDE_WEB_PORT"
echo "WireGuard is running on port $WG_PORT"
