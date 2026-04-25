use soroban_sdk::{symbol_short, Address, Env};

use crate::storage_types::{bump_instance, DataKey};

// --- Core admin storage helpers ---

pub fn read_admin(e: &Env) -> Address {
    bump_instance(e);
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn write_admin(e: &Env, id: &Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::Admin, id);
}

pub fn has_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

/// Verifies that `admin` is the current admin and has authorized the call.
pub fn check_admin(e: &Env, admin: &Address) {
    admin.require_auth();
    let stored = read_admin(e);
    if admin != &stored {
        panic!("not authorized: caller is not the admin");
    }
}

/// Rotates the stored admin to `new_admin`. Must be called by the current admin.
pub fn transfer_admin(e: &Env, new_admin: Address) {
    let current_admin = read_admin(e);
    current_admin.require_auth();
    write_admin(e, &new_admin);
    e.events().publish(
        (symbol_short!("admin_set"), current_admin),
        new_admin,
    );
}
