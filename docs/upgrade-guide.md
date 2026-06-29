# Contract Upgrade Guide

This document covers how to safely upgrade the Veritix Pay Soroban contract on-chain, including preparation, execution, and rollback.

---

## How Soroban Contract Upgrades Work

Soroban upgrades work via **WASM hash replacement**. The contract's code (WASM binary) is identified by a hash stored in the ledger. Calling `update_current_contract_wasm` replaces that hash with a new WASM upload, taking effect immediately on the next invocation.

Key properties:
- The **contract address does not change** — all existing integrations remain valid.
- **All persistent and instance storage is preserved** — no data is wiped.
- The **WASM bytecode is replaced** — new function signatures, logic, and module changes apply immediately.
- Only the contract's own admin can authorize an upgrade (enforced by `admin.rs`).

---

## What Is Preserved vs. Reset

| Item | Preserved? | Notes |
|------|-----------|-------|
| Contract address | ✅ Yes | Never changes |
| All storage entries (`DataKey` variants) | ✅ Yes | Persistent and instance storage survive |
| Admin address | ✅ Yes | Stored under `DataKey::Admin` |
| Token balances, escrows, disputes, etc. | ✅ Yes | All `DataKey` entries intact |
| WASM bytecode | ❌ Replaced | New binary takes effect immediately |
| In-flight transaction results | N/A | Transactions either committed or not before upgrade |

---

## Preparing a Migration

### 1. Audit Storage Layout Changes

Compare `storage_types.rs` between the old and new version:

- **Added keys** — safe; old storage simply won't have them until first write.
- **Removed keys** — safe to remove from code, but orphaned ledger entries will remain (no automatic cleanup).
- **Renamed/retyped keys** — **breaking**; old values will be read as the new type, causing deserialization panics. You must migrate data before removing the old key variant.

Refer to [`docs/storage-layout.md`](./storage-layout.md) for the authoritative list of keys and their types.

### 2. Test on Testnet

```bash
# Build the new WASM
cd veritixpay/contract/token
make build

# Deploy to testnet and run invoke smoke tests
stellar contract deploy \
  --wasm target/wasm32v1-none/release/veritix_token.wasm \
  --network testnet \
  --source <ADMIN_KEYPAIR>

# Run the full test suite against testnet
make test
```

Verify all existing function signatures still work and any new entry points behave correctly with live storage.

### 3. Checklist Before Upgrade

- [ ] All added/removed/changed `DataKey` variants are documented in `docs/storage-layout.md`
- [ ] New WASM compiled with `make build` (release profile, `wasm32v1-none` target)
- [ ] Full test suite passes locally (`make test`)
- [ ] Smoke-tested on testnet with real storage state
- [ ] Admin key is available and confirmed
- [ ] Team has reviewed the diff — get a second pair of eyes on any storage layout change

---

## Step-by-Step Upgrade Procedure

### Step 1 — Build the New WASM

```bash
cd veritixpay/contract/token
make build
# Output: target/wasm32v1-none/release/veritix_token.wasm
```

### Step 2 — Upload the WASM to the Network

```bash
stellar contract upload \
  --wasm target/wasm32v1-none/release/veritix_token.wasm \
  --network mainnet \
  --source <ADMIN_KEYPAIR>
# Outputs a NEW_WASM_HASH
```

Record the `NEW_WASM_HASH` printed by this command.

### Step 3 — Invoke the Upgrade

Call `upgrade` on the live contract. This requires admin authorization:

```bash
stellar contract invoke \
  --id <CONTRACT_ADDRESS> \
  --network mainnet \
  --source <ADMIN_KEYPAIR> \
  -- upgrade \
  --new_wasm_hash <NEW_WASM_HASH>
```

The contract's WASM is replaced atomically when the transaction is confirmed.

### Step 4 — Verify

```bash
# Confirm the WASM hash on-chain matches what you uploaded
stellar contract info --id <CONTRACT_ADDRESS> --network mainnet

# Run a smoke-test invocation against the live contract
stellar contract invoke \
  --id <CONTRACT_ADDRESS> \
  --network mainnet \
  --source <ADMIN_KEYPAIR> \
  -- name
```

---

## Rollback Strategy

Soroban does not have a built-in "undo" for upgrades, but rollback is achievable within the **ledger entry TTL window**:

1. **Keep the old WASM hash** — note it before upgrading (visible in `stellar contract info`).
2. The old WASM binary remains in the ledger as long as its TTL has not expired (ledger entries expire; uploaded WASMs have a TTL just like other entries).
3. To roll back, re-upload the old WASM binary (or reference the existing hash if it hasn't expired) and call `upgrade` again with the old hash.

```bash
# Re-upload the previous WASM if needed
stellar contract upload \
  --wasm path/to/old/veritix_token.wasm \
  --network mainnet \
  --source <ADMIN_KEYPAIR>

# Upgrade back to the old hash
stellar contract invoke \
  --id <CONTRACT_ADDRESS> \
  --network mainnet \
  --source <ADMIN_KEYPAIR> \
  -- upgrade \
  --new_wasm_hash <OLD_WASM_HASH>
```

> **Important:** Storage written by the new WASM version (e.g., new keys) will remain after rolling back. Ensure the old WASM handles any such entries gracefully (typically by ignoring unknown keys).

---

## Per-Field Change Documentation

Every storage layout change in an upgrade **must be documented here** before the upgrade is submitted.

| Upgrade | Key | Change | Migration Required? |
|---------|-----|--------|-------------------|
| v1.0 → v1.1 | *(example)* `Pause` bool added | New key — no migration | No |

Add a row to this table for each `DataKey` variant added, removed, or retyped in the upgrade.
