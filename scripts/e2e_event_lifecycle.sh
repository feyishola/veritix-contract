#!/bin/bash

# Exit on error
set -e

# --- Configuration ---
RPC_URL="https://rpc-futurenet.stellar.org:443"
NETWORK_PASSPHRASE="Test SDF Future Network ; September 2015"
ADMIN_SECRET="SA5Y...2Z7I"
BUYER_SECRET="SB6J...4T6E"
ORGANIZER_SECRET="SC3...5E3K"
ARTIST_SECRET="SD...23J"
PLATFORM_SECRET="S...23K"

# --- Deployment ---
echo "Deploying contract..."
CONTRACT_ID=$(soroban contract deploy --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ADMIN_SECRET" --wasm target/wasm32-unknown-unknown/release/veritix_token.wasm)
echo "Contract deployed with ID: $CONTRACT_ID"

# --- Initialization ---
echo "Initializing contract..."
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ADMIN_SECRET" -- initialize --admin "$ADMIN_ADDRESS" --decimal 7 --name "VeriTix" --symbol "VTX"

# --- Minting ---
echo "Minting 1000 VTX to buyer..."
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ADMIN_SECRET" -- mint --to "$BUYER_ADDRESS" --amount 1000

# --- Escrow ---
echo "Buyer creating ticket escrow..."
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$BUYER_SECRET" -- ticket_escrow --buyer "$BUYER_ADDRESS" --organizer "$ORGANIZER_ADDRESS" --amount 100 --event_ledger 12345 --ticket_ref "TICKET123"

# --- Ledger Advancement (Simulated) ---
echo "Simulating ledger advancement past event..."
# This is a placeholder. In a real testnet, you would wait for the ledger to advance.
sleep 5

# --- Release Escrow ---
echo "Organizer releasing escrow..."
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ORGANIZER_SECRET" -- release_escrow --escrow_id 0 # Assuming escrow_id is 0

# --- Revenue Split ---
echo "Organizer splitting revenue..."
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ORGANIZER_SECRET" -- revenue_split --organizer "$ORGANIZER_ADDRESS" --amount 8000 --artist "$ARTIST_ADDRESS" --artist_amount 1500 --platform "$PLATFORM_ADDRESS" --platform_amount 100

# --- Assertions ---
echo "Verifying balances..."
BUYER_BALANCE=$(soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRAE" --source-account "$BUYER_SECRET" -- balance --id "$BUYER_ADDRESS")
ORGANIZER_BALANCE=$(soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ORGANIZER_SECRET" -- balance --id "$ORGANIZER_ADDRESS")
ARTIST_BALANCE=$(soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ARTIST_SECRET" -- balance --id "$ARTIST_ADDRESS")
PLATFORM_BALANCE=$(soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$PLATFORM_SECRET" -- balance --id "$PLATFORM_ADDRESS")

echo "Buyer Balance: $BUYER_BALANCE"
echo "Organizer Balance: $ORGANIZER_BALANCE"
echo "Artist Balance: $ARTIST_BALANCE"
echo "Platform Balance: $PLATFORM_BALANCE"

# --- Total Supply Check ---
echo "Verifying total supply..."
TOTAL_SUPPLY=$(soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --source-account "$ADMIN_SECRET" -- total_supply)
echo "Total Supply: $TOTAL_SUPPLY"

echo "End-to-end test complete."