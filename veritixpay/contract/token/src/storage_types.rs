//! Shared storage model.
//! Defines instance-vs-persistent key ownership, value shapes, and TTL bump policy constants.
//! Instance storage is for contract-global config/counters; persistent storage is for user/payment records.

use soroban_sdk::{contracttype, Address, Env, IntoVal, TryFromVal, Val};

pub const BALANCE_LIFETIME_THRESHOLD: u32 = 518400; // ~30 days
pub const BALANCE_BUMP_AMOUNT: u32 = 535000;
pub const ALLOWANCE_LIFETIME_THRESHOLD: u32 = 518400; // ~30 days
pub const ALLOWANCE_BUMP_AMOUNT: u32 = 535000;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 518400;
pub const INSTANCE_BUMP_AMOUNT: u32 = 535000;
/// Threshold and bump for long-lived persistent records (escrow, split, dispute, recurring, freeze).
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 518400; // ~30 days
pub const PERSISTENT_BUMP_AMOUNT: u32 = 535000;
pub const SPLIT_LIFETIME_THRESHOLD: u32 = PERSISTENT_LIFETIME_THRESHOLD;
pub const SPLIT_BUMP_AMOUNT: u32 = PERSISTENT_BUMP_AMOUNT;
pub const RECURRING_LIFETIME_THRESHOLD: u32 = PERSISTENT_LIFETIME_THRESHOLD;
pub const RECURRING_BUMP_AMOUNT: u32 = PERSISTENT_BUMP_AMOUNT;
pub const DISPUTE_LIFETIME_THRESHOLD: u32 = PERSISTENT_LIFETIME_THRESHOLD;
pub const DISPUTE_BUMP_AMOUNT: u32 = PERSISTENT_BUMP_AMOUNT;
pub const ESCROW_LIFETIME_THRESHOLD: u32 = 7_884_000;
pub const ESCROW_BUMP_AMOUNT: u32 = 7_900_000;

#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Allowance(AllowanceDataKey),
    SpenderAllowances(Address),
    Balance(Address),
    Metadata,
    TotalSupply,
    EscrowCount,
    Escrow(u32),
    RecurringCount,
    Recurring(u32),
    SplitCount,
    Split(u32),
    DisputeCount,
    Dispute(u32),
    // Tracks the active dispute ID for a given escrow (None = no open dispute).
    EscrowDispute(u32),

    // Stores per-address freeze status.
    Freeze(Address),
}

pub fn read_persistent_record<T>(e: &Env, key: &DataKey, missing_message: &'static str) -> T
where
    T: TryFromVal<Env, Val>,
{
    let storage = e.storage().persistent();
    let value = storage
        .get::<DataKey, T>(key)
        .unwrap_or_else(|| panic!("{}", missing_message));
    storage.extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    value
}

pub fn write_persistent_record<T>(e: &Env, key: &DataKey, value: &T)
where
    T: IntoVal<Env, Val>,
{
    let storage = e.storage().persistent();
    storage.set(key, value);
    storage.extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

/// Bumps the instance storage TTL. Call this on any entrypoint that reads or
/// writes instance-stored data (admin, metadata, counters, total supply).
pub fn bump_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn read_counter(e: &Env, key: &DataKey) -> u32 {
    e.storage().instance().get(key).unwrap_or(0)
}

pub fn increment_counter(e: &Env, key: &DataKey) -> u32 {
    // Keep instance storage alive whenever counters are mutated to prevent ID reset/collision.
    bump_instance(e);
    let next = read_counter(e, key) + 1;
    e.storage().instance().set(key, &next);
    next
}
