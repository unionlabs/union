#!/bin/bash

set -e

SERVICE_USER=prysm-beacon

# Create the service account, if needed
getent passwd $SERVICE_USER > /dev/null || useradd -s /bin/false --no-create-home --system --user-group $SERVICE_USER

# Create directories
mkdir -p /etc/prysm
mkdir -p /var/lib/prysm
install -d -m 0700 -o $SERVICE_USER -g $SERVICE_USER /var/lib/prysm/beacon-chain