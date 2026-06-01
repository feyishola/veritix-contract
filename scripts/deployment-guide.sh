#!/usr/bin/env bash
set -euo pipefail

cat <<'EOF'
Deployment Guide

Prerequisites
- A funded Stellar account
- Stellar CLI installed and configured
- Rust and the wasm32v1-none target installed

Environment Variables
- STELLAR_NETWORK: testnet or localnet
- STELLAR_ACCOUNT: Stellar account alias or secret key

Testnet Deployment
1. Generate or configure a key:
   stellar keys generate alice --network testnet
2. Fund the account with Friendbot or your preferred testnet faucet.
3. Build the contract:
   make build
4. Deploy and initialize the contract:
   STELLAR_NETWORK=testnet STELLAR_ACCOUNT=alice ./scripts/deploy.sh

Contract Initialization
- admin: deployer address
- name: VeritixToken
- symbol: VTX
- decimal: 7

Invoke Examples
- CONTRACT_ID=C... ./scripts/invoke.sh mint alice bob 1000
- CONTRACT_ID=C... ./scripts/invoke.sh transfer alice bob 100
- CONTRACT_ID=C... ./scripts/invoke.sh balance alice
- CONTRACT_ID=C... ./scripts/invoke.sh approve alice spender 50 1000000

Mainnet Checklist
- Confirm audit status before deployment
- Protect the admin key with stronger operational controls
- Keep a backup plan for contract upgrades and recovery
- Verify the network target before running the deploy script
EOF
