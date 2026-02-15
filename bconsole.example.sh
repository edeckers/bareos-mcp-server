#!/usr/bin/env bash
# Example wrapper script for bconsole
# Copy this to 'bconsole' and customize for your setup

# Option 1: Local bconsole
# bconsole -c /etc/bareos/bconsole.conf "$@"

# Option 2: Remote bconsole via SSH
# Replace 'your-bareos-host' with your actual hostname
ssh your-bareos-host "sudo bconsole $*"

# Option 3: Remote bconsole with specific user
# ssh user@your-bareos-host "sudo bconsole $*"

# Option 4: Remote bconsole without sudo (if user has permissions)
# ssh your-bareos-host "bconsole $*"
