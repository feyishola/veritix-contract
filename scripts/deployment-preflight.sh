#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking Stellar CLI"
stellar --version

echo "==> Checking Rust"
rustc --version

echo "==> Checking wasm32v1-none target"
rustup target list --installed | grep -q '^wasm32v1-none$'

echo "==> Deployment environment"
echo "STELLAR_NETWORK=${STELLAR_NETWORK:-testnet}"
echo "STELLAR_ACCOUNT=${STELLAR_ACCOUNT:-<required>}"

echo "Deployment preflight passed."
