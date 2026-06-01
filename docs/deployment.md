# Deployment Guide

This guide covers the repository's standard deploy flow for the Veritix token contract.

## Prerequisites

- A funded Stellar account
- The Stellar CLI installed and configured
- Rust and the `wasm32v1-none` target installed

## Environment Variables

- `STELLAR_NETWORK`: `testnet` or `localnet` (defaults to `testnet` in `scripts/deploy.sh`)
- `STELLAR_ACCOUNT`: Stellar account alias or secret key used as the deploy source

## Testnet Deployment

1. Generate or configure a key:

```bash
stellar keys generate alice --network testnet
```

2. Fund the account with Friendbot from the Stellar testnet tooling or your preferred testnet faucet.
3. Build the contract:

```bash
make build
```

4. Deploy and initialize the contract:

```bash
STELLAR_NETWORK=testnet STELLAR_ACCOUNT=alice ./scripts/deploy.sh
```

The deploy script will:

- build the WASM artifact
- deploy the contract
- write the resulting contract ID to `.contract-id`
- initialize the contract with the admin address, token name, symbol, and decimals

## Contract Initialization

The deploy flow calls `initialize` with:

- the deployer address as `admin`
- `VeritixToken` as the token name
- `VTX` as the symbol
- `7` as the decimal precision

## Using `make invoke`

The `scripts/invoke.sh` helper reads `.contract-id` by default. Common examples:

```bash
CONTRACT_ID=C... ./scripts/invoke.sh mint alice bob 1000
CONTRACT_ID=C... ./scripts/invoke.sh transfer alice bob 100
CONTRACT_ID=C... ./scripts/invoke.sh balance alice
CONTRACT_ID=C... ./scripts/invoke.sh approve alice spender 50 1000000
```

If you prefer to set an environment variable:

```bash
export CONTRACT_ID=C...
./scripts/invoke.sh balance alice
```

## Mainnet Checklist

- Confirm audit status before deployment
- Protect the admin key with stronger operational controls
- Keep a backup plan for contract upgrades and recovery
- Verify the network target before running the deploy script

