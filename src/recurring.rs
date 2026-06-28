use soroban_sdk::{Env, Address, Vec};
use crate::storage_types::{DataKey, RecurringPayment};

pub fn execute_recurring(e: Env, caller: Address, recurring_id: u32, amount: i128) {
    caller.require_auth();
    // stub implementation just for appending history as required
    let mut history = get_recurring_history(e.clone(), recurring_id);
    history.push_back(RecurringPayment {
        recurring_id,
        execution_ledger: e.ledger().sequence(),
        amount,
    });
    e.storage().persistent().set(&DataKey::RecurringHistory(recurring_id), &history);
}

pub fn get_recurring_history(e: Env, recurring_id: u32) -> Vec<RecurringPayment> {
    e.storage()
        .persistent()
        .get(&DataKey::RecurringHistory(recurring_id))
        .unwrap_or(Vec::new(&e))
}