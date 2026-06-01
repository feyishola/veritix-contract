#!/usr/bin/env bash
set -euo pipefail

guide_output="$(bash scripts/deployment-guide.sh)"

grep -q "Prerequisites" <<<"$guide_output"
grep -q "STELLAR_NETWORK" <<<"$guide_output"
grep -q "Testnet Deployment" <<<"$guide_output"
grep -q "Mainnet Checklist" <<<"$guide_output"

echo "deployment guide script output validated"
