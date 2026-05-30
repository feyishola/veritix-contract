use crate::balance::{receive_balance, spend_balance};
use crate::storage_types::{increment_counter, DataKey, RECURRING_BUMP_AMOUNT, RECURRING_LIFETIME_THRESHOLD};
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

/// Sets up a new recurring payment configuration.
pub fn setup_recurring(
    e: &Env,
    payer: Address,
    payee: Address,
    amount: i128,
    interval: u32,
) -> u32 {
    // 1. Validate amount and interval
    require_positive_amount(amount);

    if interval == 0 {
        panic!("interval must be positive");
    }

    if payer == payee {
        panic!("InvalidRecurring: payer and payee cannot be the same address");
    }

    // 2. Authorization: both parties must authorize the recurring charge setup.
    payer.require_auth();
    payee.require_auth();

    // 2. Increment and get the new Recurring ID
    let count = increment_counter(e, &DataKey::RecurringCount);

    // 3. Store the recurring record
    let record = RecurringRecord {
        id: count,
        payer: payer.clone(),
        payee: payee.clone(),
        amount,
        interval,
        last_charged_ledger: e.ledger().sequence(), // Set initial timestamp to now
        active: true,
    };
    let key = DataKey::Recurring(count);
    e.storage().persistent().set(&key, &record);
    e.storage()
        .persistent()
        .extend_ttl(&key, RECURRING_LIFETIME_THRESHOLD, RECURRING_BUMP_AMOUNT);

    // 4. Emit Observability Event
    e.events().publish(
        (symbol_short!("recurring_setup"), payer.clone()),
        (payee, amount),
    );

    count
}

/// Executes a recurring payment if the interval has passed.
/// Anyone can call this ("crank the contract"), but funds only move from payer to payee.
pub fn execute_recurring(e: &Env, recurring_id: u32) {
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    e.storage()
        .persistent()
        .extend_ttl(&key, RECURRING_LIFETIME_THRESHOLD, RECURRING_BUMP_AMOUNT);

    if !record.active {
        panic!("recurring payment is not active");
    }

    let current_ledger = e.ledger().sequence();
    if current_ledger < record.last_charged_ledger + record.interval {
        panic!("interval has not elapsed");
    }

    let payer_balance = crate::balance::read_balance(e, record.payer.clone());
    if payer_balance < record.amount {
        panic!(
            "InsufficientBalance: payer has insufficient balance for recurring payment {}",
            recurring_id
        );
    }

    spend_balance(e, record.payer.clone(), record.amount);
    receive_balance(e, record.payee.clone(), record.amount);

    record.last_charged_ledger = current_ledger;
    e.storage().persistent().set(&key, &record);
    e.storage()
        .persistent()
        .extend_ttl(&key, RECURRING_LIFETIME_THRESHOLD, RECURRING_BUMP_AMOUNT);

    e.events().publish(
        (symbol_short!("recurring_executed"), recurring_id),
        record.amount,
    );
}

/// Cancels a recurring payment. Only the payer can cancel.
pub fn cancel_recurring(e: &Env, caller: Address, recurring_id: u32) {
    caller.require_auth();

    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));

    if record.payer != caller {
        panic!("unauthorized");
    }

    record.active = false;
    e.storage().persistent().set(&key, &record);
    e.storage()
        .persistent()
        .extend_ttl(&key, RECURRING_LIFETIME_THRESHOLD, RECURRING_BUMP_AMOUNT);

    e.events().publish(
        (
            symbol_short!("recurring_cancelled"),
            recurring_id,
            caller.clone(),
        ),
        (),
    );
}

pub fn get_recurring(e: &Env, recurring_id: u32) -> RecurringRecord {
    let key = DataKey::Recurring(recurring_id);
    let record = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    e.storage()
        .persistent()
        .extend_ttl(&key, RECURRING_LIFETIME_THRESHOLD, RECURRING_BUMP_AMOUNT);
    record
}
