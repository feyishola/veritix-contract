use crate::storage_types::{DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{symbol_short, Address, Env};

pub fn is_frozen(e: &Env, addr: &Address) -> bool {
    let key = DataKey::Freeze(addr.clone());
    let storage = e.storage().persistent();
    let frozen = storage.get(&key).unwrap_or(false);
    if frozen {
        storage.extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }
    frozen
}

pub fn freeze_account(e: &Env, _admin: Address, target: Address) {
    let admin = _admin;
    let key = DataKey::Freeze(target.clone());
    let storage = e.storage().persistent();
    storage.set(&key, &true);
    storage.extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("frozen"), target), admin);
}

pub fn unfreeze_account(e: &Env, _admin: Address, target: Address) {
    let admin = _admin;
    e.storage()
        .persistent()
        .remove(&DataKey::Freeze(target.clone()));
    e.events()
        .publish((symbol_short!("unfrozen"), target), admin);
}
