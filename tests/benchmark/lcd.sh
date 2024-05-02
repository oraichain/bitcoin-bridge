#!/bin/bash

# Check if API endpoint is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <API_ENDPOINT>"
    exit 1
fi

# API endpoint
API_ENDPOINT="$1"

# Function to run autocannon with specified connections
run_autocannon() {
    echo "Running autocannon with $1 connections..."
    autocannon -c $1 $API_ENDPOINT
    echo "Autocannon finished with $1 connections."
}

# For depositing and withdrawing, we mainly use 
# 1. checkpoint_data endpoint
# 2. deposit_fee endpoint
# 3. withdrawal_fee endpoint
# 4. fee_info endpoint
# 5. total_value_locked endpoint
# 6. escrow_balance endpoint
# 7. checkpoint_queue endpoint

# Below are script for benchmarking
# Environment: Macbook Pro - Chip: Apple M3 Pro - Ram: 18G
# 10 connections
run_autocannon 10
# 100 connections
run_autocannon 100
# 1000 connections
run_autocannon 1000
# 2000 connections
run_autocannon 2000
# 5000 connections
run_autocannon 5000
# 10000 connections
run_autocannon 10000

