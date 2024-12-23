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

# Restart polkit to apply changes
echo "Restarting polkit service to apply changes..."
sudo systemctl restart polkit

echo "Setup complete. Log out and log back in for group changes to take effect."
