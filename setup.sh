#!/bin/bash

# Evolution Simulation - Main Setup Script
# Delegates to scripts in the scripts/ folder

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Call the actual setup script in the scripts folder
exec "$SCRIPT_DIR/scripts/setup.sh" "$@" 