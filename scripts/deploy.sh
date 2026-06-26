#!/usr/bin/env bash
# Deploy VeritixToken to localnet or testnet.
#
# Required environment variables:
#   STELLAR_NETWORK  — "localnet" or "testnet" (default: testnet)
#   STELLAR_ACCOUNT  — Stellar account alias or secret key (required)
#
# Prerequisites:
#   stellar CLI installed and configured (https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)
#   Run: stellar keys generate alice --network testnet   (or localnet)

set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
ACCOUNT="${STELLAR_ACCOUNT:?STELLAR_ACCOUNT is required}"
CONTRACT_DIR="$(cd "$(dirname "$0")/../veritixpay/contract/token" && pwd)"
WASM_PATH="$(cd "$(dirname "$0")/.." && pwd)/target/wasm32v1-none/release/veritixpay.wasm"

echo "==> Building contract..."
(cd "$CONTRACT_DIR" && stellar contract build)

echo "==> Deploying to $NETWORK as $ACCOUNT..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source "$ACCOUNT" \
  --network "$NETWORK")

echo "Contract deployed: $CONTRACT_ID"
echo "$CONTRACT_ID" > .contract-id
echo "Saved contract id to .contract-id"

echo "==> Initializing contract..."
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ACCOUNT" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$(stellar keys address "$ACCOUNT" --network "$NETWORK")" \
  --name "VeritixToken" \
  --symbol "VTX" \
  --decimal 7

echo "Contract initialized."
echo "CONTRACT_ID=$CONTRACT_ID"
