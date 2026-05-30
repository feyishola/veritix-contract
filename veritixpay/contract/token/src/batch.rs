use crate::admin::check_admin;
use crate::balance::{increase_supply, receive_balance};
use crate::validation::require_positive_amount;
use soroban_sdk::{contracttype, symbol_short, Address, Env, Vec};

/// Represents a single entry in a batch mint.
#[contracttype]
#[derive(Clone)]
pub struct BatchEntry {
    pub address: Address,
    pub amount: i128,
}

/// Admin mints tokens to multiple recipients in one call.
/// Maximum 50 recipients per batch.
pub fn mint_batch(e: &Env, admin: Address, recipients: Vec<BatchEntry>) {
    check_admin(e, &admin);
    if recipients.len() > 50 {
        panic!("BatchLimit: maximum 50 recipients per call");
    }
    let mut total: i128 = 0;
    for entry in recipients.iter() {
        require_positive_amount(entry.amount);
        receive_balance(e, entry.address.clone(), entry.amount);
        increase_supply(e, entry.amount);
        total = total.checked_add(entry.amount).expect("overflow");
    }
    e.events().publish((symbol_short!("batch_mint"), admin), total);
}
