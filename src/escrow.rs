use soroban_sdk::{contracttype, token, Address, Bytes, Env, Vec};
use crate::storage_types::DataKey;


use crate::storage_types::MAX_ESCROW_AMOUNT;

pub fn create_escrow(
    e: Env,
    depositor: Address,
    beneficiary: Address,
    token: Address,
    amount: i128,
    expiry_ledger: u32,
    memo: Bytes,
) -> u32 {
    // 1. Existing baseline validation checks
    // require_positive_amount(amount);

    // 2. Supply Caps Rule: Enforce ceiling boundary to protect total liquidity pools
    if amount > MAX_ESCROW_AMOUNT {
        panic!("AmountTooLarge: use multi-party escrow for large amounts");
    }

    // Proceed with contract state allocation and structural storage...
}

#[contracttype]
#[derive(Clone)]
pub struct EscrowRecord {
    pub id: u32,
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,           // original locked amount — never changes
    pub released_amount: i128,  // #174: how much has been released so far
    pub expiry_ledger: u32,
    pub released: bool,         // true only when fully released
    pub refunded: bool,
    pub memo: Bytes,            // #175: arbitrary tag — max 64 bytes
}

// Anti-spam configuration threshold (5 minutes cooldown window)
const ESCROW_COOLDOWN_SECONDS: u64 = 300;

// ── Storage helpers ──────────────────────────────────────────────────────────

fn read_escrow_ids(e: &Env, key: DataKey) -> Vec<u32> {
    e.storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(e))
}

fn append_escrow_id(e: &Env, key: DataKey, id: u32) {
    let mut list = read_escrow_ids(e, key.clone());
    list.push_back(id);
    e.storage().persistent().set(&key, &list);
}

fn load_record(e: &Env, escrow_id: u32) -> EscrowRecord {
    e.storage()
        .persistent()
        .get(&DataKey::Escrow(escrow_id))
        .expect("escrow not found")
}

fn save_record(e: &Env, record: &EscrowRecord) {
    e.storage()
        .persistent()
        .set(&DataKey::Escrow(record.id), record);
}

fn get_admin(e: &Env) -> Address {
    e.storage()
        .persistent()
        .get(&DataKey::Admin)
        .expect("admin not set")
}

// ── Public functions ─────────────────────────────────────────────────────────

/// Create an escrow. 
/// #175 enforces `memo: Bytes` — max 64 bytes.
/// #269 enforces dynamic rate limiting based on block timestamp history.
pub fn create_escrow(
    e: Env,
    depositor: Address,
    beneficiary: Address,
    token_addr: Address,
    amount: i128,
    expiry_ledger: u32,
    memo: Bytes,
) -> u32 {
    depositor.require_auth();

    // #269: Strict Anti-Spam Rate Limiting Guard Check
    let rate_limit_key = DataKey::LastEscrowTime(depositor.clone());
    let last_creation_time: u64 = e.storage().persistent().get(&rate_limit_key).unwrap_or(0);
    let current_time = e.ledger().timestamp();

    if last_creation_time > 0 && (current_time - last_creation_time) < ESCROW_COOLDOWN_SECONDS {
        panic!("RateLimitExceeded: please wait before creating another escrow");
    }

    // #175: enforce memo length limit with the exact panic string required
    if memo.len() > 64 {
        panic!("MemoTooLong: memo cannot exceed 64 bytes");
    }

    assert!(amount > 0, "amount must be greater than zero");
    assert!(
        expiry_ledger > e.ledger().sequence(),
        "expiry_ledger must be in the future"
    );

    let id: u32 = e
        .storage()
        .persistent()
        .get(&DataKey::EscrowCount)
        .unwrap_or(0);

    // Pull tokens from depositor into the contract
    let token_client = token::Client::new(&e, &token_addr);
    token_client.transfer(&depositor, &e.current_contract_address(), &amount);

    let record = EscrowRecord {
        id,
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token_addr,
        amount,
        released_amount: 0, // #174: starts at zero
        expiry_ledger,
        released: false,
        refunded: false,
        memo,               // #175
    };

    save_record(&e, &record);
    append_escrow_id(&e, DataKey::DepositorEscrows(depositor.clone()), id);
    append_escrow_id(&e, DataKey::BeneficiaryEscrows(beneficiary), id);

    // Update state tracking counters and rate limit timestamps cleanly
    e.storage().persistent().set(&DataKey::EscrowCount, &(id + 1));
    e.storage().persistent().set(&rate_limit_key, &current_time);

    // #181: emit escrow_created event with memo for indexers
    e.events().publish(
        (
            soroban_sdk::symbol_short!("escrow_cre"),
            record.depositor.clone(),
            record.beneficiary.clone(),
        ),
        (record.amount, record.memo.clone()),
    );

    id
}

/// Full release — sends everything remaining to the beneficiary.
pub fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
    caller.require_auth();

    let mut record = load_record(&e, escrow_id);

    assert!(!record.released, "already released");
    assert!(!record.refunded, "already refunded");
    assert!(
        caller == record.depositor || caller == get_admin(&e),
        "not authorised to release"
    );
    assert!(
        e.ledger().sequence() <= record.expiry_ledger,
        "escrow has expired"
    );

    let remaining = record.amount - record.released_amount;
    assert!(remaining > 0, "nothing left to release");

    record.released_amount = record.amount;
    record.released = true;
    save_record(&e, &record);

    let token_client = token::Client::new(&e, &record.token);
    token_client.transfer(&e.current_contract_address(), &record.beneficiary, &remaining);

    // #181: emit escrow_released event with memo for indexers
    e.events().publish(
        (
            soroban_sdk::symbol_short!("escrow_rel"),
            record.depositor.clone(),
            record.beneficiary.clone(),
        ),
        (remaining, record.memo.clone()),
    );
}

/// #174: Partial release — caller must be the beneficiary.
pub fn release_partial_escrow(e: Env, caller: Address, escrow_id: u32, amount: i128) {
    caller.require_auth();

    let mut record = load_record(&e, escrow_id);

    assert!(!record.refunded, "already refunded");
    assert!(!record.released, "already fully released");
    assert!(
        caller == record.beneficiary,
        "only the beneficiary can partially release"
    );
    assert!(
        e.ledger().sequence() <= record.expiry_ledger,
        "escrow has expired"
    );
    assert!(amount > 0, "release amount must be greater than zero");

    let remaining = record.amount - record.released_amount;
    assert!(
        amount <= remaining,
        "release amount exceeds remaining balance"
    );

    record.released_amount += amount;

    // Mark fully released if nothing is left
    if record.released_amount == record.amount {
        record.released = true;
    }

    save_record(&e, &record);

    let token_client = token::Client::new(&e, &record.token);
    token_client.transfer(&e.current_contract_address(), &record.beneficiary, &amount);
}

/// Refund — returns original locked amount minus what was already partially released.
pub fn refund_escrow(e: Env, caller: Address, escrow_id: u32) {
    caller.require_auth();

    let mut record = load_record(&e, escrow_id);

    assert!(!record.released, "already released");
    assert!(!record.refunded, "already refunded");
    assert!(
        caller == record.depositor || caller == get_admin(&e),
        "not authorised to refund"
    );

    let refundable = record.amount - record.released_amount;
    assert!(refundable > 0, "nothing left to refund");

    record.refunded = true;
    save_record(&e, &record);

    let token_client = token::Client::new(&e, &record.token);
    token_client.transfer(
        &e.current_contract_address(),
        &record.depositor,
        &refundable,
    );

    // #181: emit escrow_refunded event with memo for indexers
    e.events().publish(
        (
            soroban_sdk::symbol_short!("escrow_ref"),
            record.depositor.clone(),
            record.beneficiary.clone(),
        ),
        (refundable, record.memo.clone()),
    );
}

// ── Query helpers ─────────────────────────────────────────────────────────────

pub fn get_escrows_by_depositor(e: Env, depositor: Address) -> Vec<u32> {
    read_escrow_ids(&e, DataKey::DepositorEscrows(depositor))
}

pub fn get_escrows_by_beneficiary(e: Env, beneficiary: Address) -> Vec<u32> {
    read_escrow_ids(&e, DataKey::BeneficiaryEscrows(beneficiary))
}

pub fn get_escrowed_total(e: &Env) -> i128 {
    let escrow_count: u32 = e
        .storage()
        .persistent()
        .get(&DataKey::EscrowCount)
        .unwrap_or(0);

    let mut total = 0_i128;
    for escrow_id in 0..escrow_count {
        let record: EscrowRecord = e
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        if !record.released && !record.refunded {
            total += record.amount - record.released_amount;
        }
    }

    total
}