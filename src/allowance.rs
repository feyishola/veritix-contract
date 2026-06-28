use soroban_sdk::{contracttype, Address, Env};
use crate::storage_types::DataKey;

#[contracttype]
#[derive(Clone)]
pub struct Allowance {
    pub amount: i128,
    pub expiration_ledger: u32,
}

pub fn create_allowance(e: &Env, from: &Address, spender: &Address, amount: i128, expiration_ledger: u32) {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    let allowance = Allowance {
        amount,
        expiration_ledger,
    };
    e.storage().persistent().set(&key, &allowance);
}

pub fn read_allowance(e: &Env, from: &Address, spender: &Address) -> Allowance {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    e.storage().persistent().get(&key).unwrap_or(Allowance {
        amount: 0,
        expiration_ledger: 0,
    })
}

pub fn spend_allowance(e: &Env, from: &Address, spender: &Address, amount: i128) {
    let allowance = read_allowance(e, from, spender);
    if allowance.expiration_ledger < e.ledger().sequence() {
        panic!("allowance expired");
    }
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
    let key = DataKey::Allowance(from.clone(), spender.clone());
    let new_allowance = Allowance {
        amount: allowance.amount - amount,
        expiration_ledger: allowance.expiration_ledger,
    };
    e.storage().persistent().set(&key, &new_allowance);
}