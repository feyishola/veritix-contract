use crate::balance::{receive_balance, spend_balance};
use crate::storage_types::{
    increment_counter, DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD,
};
use crate::validation::require_positive_amount;
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringRecord {
    pub id: u32,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub interval: u32,
    pub last_charged_ledger: u32,
    pub active: bool,
}

pub fn setup_recurring(e: &Env, payer: Address, payee: Address, amount: i128, interval: u32) -> u32 {
    require_positive_amount(amount);
    if interval == 0 { panic!("interval must be positive"); }
    if payer == payee { panic!("InvalidRecurring: payer and payee cannot be the same address"); }
    payer.require_auth();
    payee.require_auth();
    let count = increment_counter(e, &DataKey::RecurringCount);
    let record = RecurringRecord {
        id: count, payer: payer.clone(), payee: payee.clone(), amount, interval,
        last_charged_ledger: e.ledger().sequence(), active: true,
    };
    let key = DataKey::Recurring(count);
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_setup"), payer.clone()), (payee, amount));
    count
}

pub fn amend_recurring(e: &Env, caller: Address, recurring_id: u32, new_amount: Option<i128>, new_interval: Option<u32>) {
    caller.require_auth();
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key).unwrap_or_else(|| panic!("recurring record not found"));
    if record.payer != caller { panic!("unauthorized"); }
    if !record.active { panic!("recurring payment is not active"); }
    if let Some(amt) = new_amount {
        require_positive_amount(amt);
        record.amount = amt;
    }
    if let Some(ivl) = new_interval {
        if ivl == 0 { panic!("interval must be >= 1"); }
        record.interval = ivl;
    }
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_amended"), recurring_id), (record.amount, record.interval));
}

pub fn execute_recurring(e: &Env, recurring_id: u32) {
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key).unwrap_or_else(|| panic!("recurring record not found"));
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    if !record.active { panic!("recurring payment is not active"); }
    let current_ledger = e.ledger().sequence();
    if current_ledger < record.last_charged_ledger + record.interval { panic!("interval has not elapsed"); }
    let payer_balance = crate::balance::read_balance(e, record.payer.clone());
    if payer_balance < record.amount { panic!("InsufficientBalance: payer has insufficient balance for recurring payment {}", recurring_id); }
    spend_balance(e, record.payer.clone(), record.amount);
    receive_balance(e, record.payee.clone(), record.amount);
    record.last_charged_ledger = current_ledger;
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_executed"), recurring_id), record.amount);
}

pub fn cancel_recurring(e: &Env, caller: Address, recurring_id: u32) {
    caller.require_auth();
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key).unwrap_or_else(|| panic!("recurring record not found"));
    if record.payer != caller { panic!("unauthorized"); }
    record.active = false;
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_cancelled"), recurring_id, caller.clone()), ());
}

pub fn get_recurring(e: &Env, recurring_id: u32) -> RecurringRecord {
    let key = DataKey::Recurring(recurring_id);
    let record = e.storage().persistent().get(&key).unwrap_or_else(|| panic!("recurring record not found"));
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    record
}