use crate::admin::check_admin;
use crate::storage_types::DataKey;
use soroban_sdk::{symbol_short, Address, Env};

/// Returns `true` if the contract is currently paused.
pub fn is_paused(e: &Env) -> bool {
    e.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}

/// Panics if the contract is paused; call at the top of transfer/mint/burn.
pub fn require_not_paused(e: &Env) {
    if is_paused(e) {
        panic!("ContractPaused: all transfers are currently paused");
    }
}

/// Admin pauses all token transfers globally.
pub fn pause(e: &Env, admin: Address) {
    check_admin(e, &admin);
    e.storage().instance().set(&DataKey::Paused, &true);
    e.events().publish((symbol_short!("paused"), admin), ());
}

/// Admin unpauses the contract, restoring normal operation.
pub fn unpause(e: &Env, admin: Address) {
    check_admin(e, &admin);
    e.storage().instance().remove(&DataKey::Paused);
    e.events().publish((symbol_short!("unpaused"), admin), ());
}
