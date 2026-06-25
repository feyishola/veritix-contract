# Storage Layout Reference

`DataKey` is defined in `veritixpay/contract/token/src/storage_types.rs`.

## Instance Storage Keys
- `Admin` -> `Address`
  - Written in `admin.rs` (`write_admin`).
  - Owner module: `admin`.
  - TTL: instance TTL via `bump_instance`.
- `Metadata` -> `TokenMetadata`
  - Written in `metadata.rs`.
  - Owner module: `metadata`.
  - TTL: instance TTL via `bump_instance`.
- `TotalSupply` -> `i128`
  - Written in `balance.rs`.
  - Owner module: `balance`.
  - TTL: instance TTL via `bump_instance`.
- `EscrowCount` / `SplitCount` / `RecurringCount` / `DisputeCount` -> `u32`
  - Written through `increment_counter`.
  - Owner modules: escrow/splitter/recurring/dispute.
  - TTL: instance TTL bumped centrally in `increment_counter`.

## Persistent Storage Keys
- `Balance(Address)` -> `i128`
  - Owner module: `balance`.
  - TTL: `BALANCE_*` constants.
- `Allowance(AllowanceDataKey)` -> `AllowanceValue`
  - Owner module: `allowance`.
  - TTL: `ALLOWANCE_*` constants.
- `SpenderAllowances(Address)` -> `Vec<Address>`
  - Owner module: `allowance`.
  - TTL: persistent TTL constants.
- `Freeze(Address)` -> `bool`
  - Owner module: `freeze`.
  - TTL: persistent TTL constants.
- `Escrow(u32)` -> `EscrowRecord`
  - Owner module: `escrow`.
  - TTL: `ESCROW_*` constants for read bump; persistent TTL on write.
- `Split(u32)` -> `SplitRecord`
  - Owner module: `splitter`.
  - TTL: `SPLIT_*` constants.
- `Recurring(u32)` -> `RecurringRecord`
  - Owner module: `recurring`.
  - TTL: `RECURRING_*` constants.
- `Dispute(u32)` -> `DisputeRecord`
  - Owner module: `dispute`.
  - TTL: `DISPUTE_*` constants.
- `EscrowDispute(u32)` -> `u32` (active dispute id)
  - Owner module: `dispute`.
  - TTL: persistent TTL constants.

## TTL Policy Summary
- Instance keys: bumped via `bump_instance` and counter mutation helper.
- Persistent keys: bumped on read/write using module-specific constants or shared persistent defaults.
- Long-horizon escrow records use year-scale constants to reduce expiry risk.
