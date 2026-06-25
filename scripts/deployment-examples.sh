#!/usr/bin/env bash
set -euo pipefail

cat <<'EOF'
Example deployment and invocation commands:

STELLAR_NETWORK=testnet STELLAR_ACCOUNT=alice ./scripts/deploy.sh
CONTRACT_ID=C... ./scripts/invoke.sh mint alice bob 1000
CONTRACT_ID=C... ./scripts/invoke.sh transfer alice bob 100
CONTRACT_ID=C... ./scripts/invoke.sh balance alice
CONTRACT_ID=C... ./scripts/invoke.sh approve alice spender 50 1000000

If .contract-id exists, scripts/invoke.sh will use it automatically.
EOF
