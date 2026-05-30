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

    // Global emergency pause flag.
    Paused,
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
    let next = read_counter(e, key) + 1;
    e.storage().instance().set(key, &next);
    next
}
