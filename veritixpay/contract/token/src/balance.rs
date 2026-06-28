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

fn update_holder_set(e: &Env, addr: &Address) {
    let key = DataKey::HolderSet;
    let mut holders: soroban_sdk::Vec<Address> = e.storage().persistent().get(&key).unwrap_or_else(|| soroban_sdk::Vec::new(e));
    let balance = read_balance(e, addr.clone());
    if balance > 0 {
        let mut exists = false;
        for i in 0..holders.len() {
            if holders.get(i).unwrap() == *addr {
                exists = true;
                break;
            }
        }
        if !exists {
            holders.push_back(addr.clone());
        }
    } else {
        let mut updated = soroban_sdk::Vec::new(e);
        for i in 0..holders.len() {
            if holders.get(i).unwrap() != *addr {
                updated.push_back(holders.get(i).unwrap());
            }
        }
        holders = updated;
    }
    e.storage().persistent().set(&key, &holders);
}

/// Adds amount to address balance — panics on overflow
pub fn receive_balance(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr.clone());
    let current_balance = read_balance(e, addr.clone()); // TTL is extended here
    let new_balance = current_balance
        .checked_add(amount)
        .expect("balance overflow");

    e.storage().persistent().set(&key, &new_balance);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    if current_balance == 0 {
        update_holder_set(e, &addr);
    }
}

/// Subtracts amount from address balance — panics if insufficient or underflow
pub fn spend_balance(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr.clone());
    let current_balance = read_balance(e, addr.clone());

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

    if new_balance == 0 {
        update_holder_set(e, &addr);
    }
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

pub fn read_max_supply(e: &Env) -> i128 {
    bump_instance(e);
    e.storage().instance().get(&DataKey::MaxSupply).unwrap_or(0)
}

pub fn increase_supply(e: &Env, amount: i128) {
    let supply = read_total_supply(e);
    let new_supply = supply.checked_add(amount).expect("supply overflow");
    let max_supply = read_max_supply(e);
    if max_supply > 0 && new_supply > max_supply {
        panic!("SupplyCap: max supply reached");
    }
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
