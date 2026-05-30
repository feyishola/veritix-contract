//! Balance and supply module.
//! Owns per-address token balances and total supply updates.
//! `receive_balance` intentionally does not check freeze status because admin/system flows
//! (mint, escrow release, split distributions) must be able to credit frozen accounts.

use crate::storage_types::{bump_instance, DataKey, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env};

/// Returns the balance for an address, or 0 if not set
pub fn read_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    let storage = e.storage().persistent();

    if let Some(balance) = storage.get::<DataKey, i128>(&key) {
        storage.extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        balance
    } else {
        0
    }
}

/// Adds amount to address balance — panics on overflow
pub fn receive_balance(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr.clone());
    let current_balance = read_balance(e, addr); // TTL is extended here
    let new_balance = current_balance
        .checked_add(amount)
        .expect("balance overflow");

    e.storage().persistent().set(&key, &new_balance);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

/// Subtracts amount from address balance — panics if insufficient or underflow
pub fn spend_balance(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr.clone());
    let current_balance = read_balance(e, addr);

    if current_balance < amount {
        panic!(
            "insufficient balance: attempted to spend {} but only {} available",
            amount, current_balance
        );
    }

    let new_balance = current_balance
        .checked_sub(amount)
        .expect("balance underflow");

    let storage = e.storage().persistent();
    storage.set(&key, &new_balance);
    storage.extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

// In veritixpay/contract/token/src/balance.rs
// (Make sure to import DataKey if not already imported)

pub fn read_total_supply(e: &Env) -> i128 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

pub fn increase_supply(e: &Env, amount: i128) {
    let supply = read_total_supply(e);
    let new_supply = supply.checked_add(amount).expect("supply overflow");
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::TotalSupply, &new_supply);
}

pub fn decrease_supply(e: &Env, amount: i128) {
    let supply = read_total_supply(e);
    if supply < amount {
        panic!("supply cannot be negative");
    }
    let new_supply = supply.checked_sub(amount).expect("supply underflow");
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::TotalSupply, &new_supply);
}
