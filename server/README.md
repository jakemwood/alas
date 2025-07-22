# Server Instructions

The server provides the reliable bonding and Icecast server. You would point your playout automation software towards the server's public IP address / Icecast
port and mount.

## Installation

1. Provision a VM at your favorite cloud provider. 1 vCPU and 1 GB seems to be sufficient, 0.5 GB seems to fail to install.
1. Get `install.sh` on the VM
1. `chmod +x install.sh`
1. `sudo ./install.sh`

Note the public key and ports for use later.

