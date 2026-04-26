use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use crate::storage_types::DataKey;

#[contracttype]
#[derive(Clone)]
pub struct EscrowRecord {
    pub id: u32,
    pub depositor: Address,
    pub beneficiary: Address,
    pub amount: i128,
    pub token: Address,
    pub expiry_ledger: u32,
    pub released: bool,
    pub refunded: bool,
}

// Helper: read a Vec<u32> list from storage, defaulting to empty
fn read_escrow_ids(e: &Env, key: DataKey) -> Vec<u32> {
    e.storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(e))
}

// Helper: append an escrow id to a stored list
fn append_escrow_id(e: &Env, key: DataKey, id: u32) {
    let mut list = read_escrow_ids(e, key.clone());
    list.push_back(id);
    e.storage().persistent().set(&key, &list);
}

pub fn create_escrow(
    e: Env,
    depositor: Address,
    beneficiary: Address,
    token: Address,
    amount: i128,
    expiry_ledger: u32,
) -> u32 {
    depositor.require_auth();

    assert!(amount > 0, "amount must be greater than zero");
    assert!(
        expiry_ledger > e.ledger().sequence(),
        "expiry_ledger must be in the future"
    );

    // Pull the current counter, default to 0
    let id: u32 = e
        .storage()
        .persistent()
        .get(&DataKey::EscrowCount)
        .unwrap_or(0);

    let record = EscrowRecord {
        id,
        depositor: depositor.clone(),
        beneficiary: beneficiary.clone(),
        token: token.clone(),
        amount,
        expiry_ledger,
        released: false,
        refunded: false,
    };

    // Transfer tokens from depositor into the contract
    let token_client = soroban_sdk::token::Client::new(&e, &token);
    token_client.transfer(&depositor, &e.current_contract_address(), &amount);

    // Store the record
    e.storage().persistent().set(&DataKey::Escrow(id), &record);

    // Update depositor index
    append_escrow_id(&e, DataKey::DepositorEscrows(depositor), id);

    // Update beneficiary index — the new part for #177
    append_escrow_id(&e, DataKey::BeneficiaryEscrows(beneficiary), id);

    // Bump the counter
    e.storage()
        .persistent()
        .set(&DataKey::EscrowCount, &(id + 1));

    id
}

pub fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
    caller.require_auth();

    let mut record: EscrowRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Escrow(escrow_id))
        .expect("escrow not found");

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

    record.released = true;
    e.storage()
        .persistent()
        .set(&DataKey::Escrow(escrow_id), &record);

    let token_client = soroban_sdk::token::Client::new(&e, &record.token);
    token_client.transfer(
        &e.current_contract_address(),
        &record.beneficiary,
        &record.amount,
    );
}

pub fn refund_escrow(e: Env, caller: Address, escrow_id: u32) {
    caller.require_auth();

    let mut record: EscrowRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Escrow(escrow_id))
        .expect("escrow not found");

    assert!(!record.released, "already released");
    assert!(!record.refunded, "already refunded");
    assert!(
        caller == record.depositor || caller == get_admin(&e),
        "not authorised to refund"
    );

    record.refunded = true;
    e.storage()
        .persistent()
        .set(&DataKey::Escrow(escrow_id), &record);

    let token_client = soroban_sdk::token::Client::new(&e, &record.token);
    token_client.transfer(
        &e.current_contract_address(),
        &record.depositor,
        &record.amount,
    );
}

pub fn get_escrows_by_depositor(e: Env, depositor: Address) -> Vec<u32> {
    read_escrow_ids(&e, DataKey::DepositorEscrows(depositor))
}

// NEW for #177 — get all escrow IDs where the given address is the beneficiary
pub fn get_escrows_by_beneficiary(e: Env, beneficiary: Address) -> Vec<u32> {
    read_escrow_ids(&e, DataKey::BeneficiaryEscrows(beneficiary))
}

fn get_admin(e: &Env) -> Address {
    e.storage()
        .persistent()
        .get(&DataKey::Admin)
        .expect("admin not set")
}
