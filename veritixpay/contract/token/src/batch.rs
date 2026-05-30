use crate::balance::{receive_balance, spend_balance};
use crate::validation::require_positive_amount;
use soroban_sdk::{contracttype, symbol_short, Address, Env, Vec};

/// Represents a single entry in a batch transfer.
#[contracttype]
#[derive(Clone)]
pub struct BatchEntry {
    pub address: Address,
    pub amount: i128,
}

/// Transfers tokens from `from` to multiple recipients in one call.
/// Maximum 50 recipients per batch.
pub fn transfer_batch(e: &Env, from: Address, recipients: Vec<BatchEntry>) {
    from.require_auth();
    if recipients.len() > 50 {
        panic!("BatchLimit: maximum 50 recipients per call");
    }
    let mut total: i128 = 0;
    for entry in recipients.iter() {
        require_positive_amount(entry.amount);
        spend_balance(e, from.clone(), entry.amount);
        receive_balance(e, entry.address.clone(), entry.amount);
        total = total.checked_add(entry.amount).expect("overflow");
    }
    e.events().publish((symbol_short!("batch_xfer"), from), total);
}
