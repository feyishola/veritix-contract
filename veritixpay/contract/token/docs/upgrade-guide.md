# Contract Upgrade Guide

## How Soroban Contract Upgrades Work

Soroban supports contract upgrades via `update_current_contract_wasm`. The WASM code
of a contract can be replaced by submitting an upgrade transaction, while the contract's
storage (instance and persistent data) is preserved across upgrades.

### WASM Hash Replacement

1. Build the new WASM blob from source.
2. Upload the WASM to the network to obtain its hash.
3. The contract calls `update_current_contract_wasm` with the new WASM hash.
4. Subsequent calls to the contract address use the new code.

## What Is Preserved vs Reset

| Component    | Preserved | Notes |
|-------------|-----------|-------|
| Instance storage | Yes | `e.storage().instance()` — admin, metadata, counters |
| Persistent storage | Yes | `e.storage().persistent()` — balances, allowances, escrows |
| WASM code | No | Replaced by the new WASM blob |
| Contract ID | Yes | The address stays the same |

## Preparing a Migration

1. **Audit storage layout changes**: Every new `DataKey` variant or struct field change
   must be documented here. Old data stored under removed keys becomes orphaned but does
   not break the contract.
2. **Test on testnet**: Deploy the current contract, apply the upgrade, and run the full
   test suite against the upgraded instance.
3. **Gas estimation**: Measure the cost of the upgrade transaction and include a buffer.

## Step-by-Step Upgrade Procedure

1. **Build new WASM**:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```
2. **Compute WASM hash** and upload to the network via Soroban CLI.
3. **Obtain admin approval**: The upgrade must be authorized by the contract admin.
4. **Submit upgrade transaction**:
   ```rust
   env.invoke_contract(&contract_id, &symbol_short!("upgrade"), (new_wasm_hash,));
   ```
   or via CLI:
   ```bash
   soroban contract invoke --id <CONTRACT_ID> -- upgrade --wasm-hash <HASH>
   ```

## Rollback Strategy

- Keep the old WASM hash after every upgrade.
- If the new code has a critical bug, resubmit an upgrade pointing back to the old hash.
- Rollback must happen within the same ledger window if state migration is backwards-compatible.

## Storage Layout Change Checklist

Every field added, removed, or renamed in `DataKey`, `EscrowRecord`, `DisputeRecord`,
`RecurringRecord`, `SplitRecord`, `AllowanceValue`, or `TokenMetadata` must be recorded
here with the issue/PR that introduced the change.

| Issue | Change | Date |
|-------|--------|------|
| — | Initial layout | — |
