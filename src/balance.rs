use soroban_sdk::Env;
use crate::storage_types::DataKey;

pub fn read_supply(e: &Env) -> i128 {
    e.storage().persistent().get(&DataKey::TotalSupply).unwrap_or(0)
}

pub fn increase_supply(e: &Env, amount: i128) {
    let supply = read_supply(e);
    let new_supply = supply.checked_add(amount).expect("supply overflow");
    e.storage().persistent().set(&DataKey::TotalSupply, &new_supply);
}

pub fn decrease_supply(e: &Env, amount: i128) {
    let supply = read_supply(e);
    let new_supply = supply.checked_sub(amount).expect("supply underflow");
    e.storage().persistent().set(&DataKey::TotalSupply, &new_supply);
}
