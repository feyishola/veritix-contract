//! Admin module.
//! Owns admin identity storage, authorization checks, and rotation events.
//! `check_admin` requires signer auth first, then identity match, to prevent spoofed caller paths.

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

pub fn read_clawback_cosigner(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::ClawbackCoSigner)
}

pub fn write_clawback_cosigner(e: &Env, cosigner: &Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::ClawbackCoSigner, cosigner);
}

pub fn read_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn propose_admin(e: &Env, new_admin: &Address) {
    let current_admin = read_admin(e);
    current_admin.require_auth();
    bump_instance(e);
    e.storage().instance().set(&DataKey::PendingAdmin, new_admin);
}

pub fn accept_admin(e: &Env) {
    let pending: Address = e.storage().instance().get(&DataKey::PendingAdmin).expect("no pending admin");
    pending.require_auth();
    let old = read_admin(e);
    write_admin(e, &pending);
    e.storage().instance().remove(&DataKey::PendingAdmin);
    e.events().publish(
        (symbol_short!("admin_set"), old),
        pending,
    );
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
