#!/bin/bash

# Check if the 'network' group exists; if not, create it
if ! getent group network >/dev/null; then
    echo "Creating 'network' group..."
    sudo groupadd network
else
    echo "'network' group already exists."
fi

# Add the currently logged-in user to the 'network' group
CURRENT_USER=$(whoami)
echo "Adding user '$CURRENT_USER' to the 'network' group..."
sudo usermod -aG network "$CURRENT_USER"

# Create the Polkit rule file for Wi-Fi scan permissions
POLKIT_RULE_FILE="/etc/polkit-1/rules.d/10-networkmanager-wifi-scan.rules"
if [ ! -f "$POLKIT_RULE_FILE" ]; then
    echo "Creating Polkit rule file at $POLKIT_RULE_FILE..."
    sudo bash -c "cat > $POLKIT_RULE_FILE" <<'EOF'
polkit.addRule(function(action, subject) {
    if (action.id.startsWith("org.freedesktop.NetworkManager") &&
        subject.isInGroup("network")) {
        return polkit.Result.YES;
    }
});
EOF
    echo "Polkit rule file created successfully."
else
    echo "Polkit rule file already exists at $POLKIT_RULE_FILE."
fi

# Create the Polkit rule file for shutdown/reboot permissions
SHUTDOWN_POLKIT_RULE_FILE="/etc/polkit-1/rules.d/50-allow-shutdown.rules"
if [ ! -f "$SHUTDOWN_POLKIT_RULE_FILE" ]; then
    echo "Creating Polkit rule file for shutdown/reboot permissions at $SHUTDOWN_POLKIT_RULE_FILE..."
    sudo bash -c "cat > $SHUTDOWN_POLKIT_RULE_FILE" <<'EOF'
polkit.addRule(function(action, subject) {
    if ((action.id == "org.freedesktop.login1.power-off" ||
         action.id == "org.freedesktop.login1.reboot") &&
        subject.isInGroup("network")) {
        return polkit.Result.YES;
    }
});
EOF
    echo "Shutdown/reboot Polkit rule file created successfully."
else
    echo "Shutdown/reboot Polkit rule file already exists at $SHUTDOWN_POLKIT_RULE_FILE."
fi

# Restart polkit to apply changes
echo "Restarting polkit service to apply changes..."
sudo systemctl restart polkit

# Install Wireguard
sudo apt install wireguard

# Install engarde-client
echo "Downloading EnGarde client..."
ENGARDE_BINARY="/usr/bin/engarde-client"
ENGARDE_URL="https://engarde.linuxzogno.org/builds/master/linux/arm/engarde-client"
sudo curl -L -o "$ENGARDE_BINARY" "$ENGARDE_URL"
sudo chmod +x "$ENGARDE_BINARY"

# Create alas folders
sudo mkdir /etc/alas
sudo mkdir /etc/alas/backups

# Setup the alas service
sudo bash -c "cat > /etc/systemd/system/alas.service" <<'EOF'
[Unit]
Description=Alas Audio Recording Service
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart=/usr/bin/alas
Restart=always
RestartSec=5
User=alas
Group=alas
WorkingDirectory=/etc/alas
StandardOutput=journal
StandardError=journal
AmbientCapabilities=CAP_NET_ADMIN
KillSignal=SIGINT

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload
sudo systemctl enable alas

echo "Setup complete. Rebooting in 3 seconds..."

sleep 3
sudo reboot

