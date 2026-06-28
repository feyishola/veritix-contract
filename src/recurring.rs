use soroban_sdk::{contracttype, token, Address, Env};
use crate::storage_types::DataKey;

#[contracttype]
#[derive(Clone)]
pub struct RecurringRecord {
    pub payer: Address,
    pub payee: Address,
    pub token: Address,
    pub amount: i128,
    pub interval: u32,
    pub last_charged_ledger: u32,
}

pub fn setup_recurring(
    e: &Env,
    payer: Address,
    payee: Address,
    token_addr: Address,
    amount: i128,
    interval: u32,
) -> u32 {
    // #426: amount must be positive — first check
    assert!(amount > 0, "amount must be positive");
    payer.require_auth();

    let id: u32 = e.storage().persistent().get(&DataKey::RecurringCount).unwrap_or(0);
    let record = RecurringRecord {
        payer,
        payee,
        token: token_addr,
        amount,
        interval,
        last_charged_ledger: e.ledger().sequence(),
    };
    e.storage().persistent().set(&DataKey::Recurring(id), &record);
    e.storage().persistent().set(&DataKey::RecurringCount, &(id + 1));
    id
}

pub fn execute_recurring(e: &Env, recurring_id: u32) {
    let mut record: RecurringRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Recurring(recurring_id))
        .expect("recurring not found");

    // #435: due-date check is FIRST — cheapest possible early exit for griefing protection
    let next_due = record.last_charged_ledger
        .checked_add(record.interval)
        .expect("overflow");
    assert!(e.ledger().sequence() >= next_due, "not yet due");

    let token_client = token::Client::new(e, &record.token);
    token_client.transfer(&record.payer, &record.payee, &record.amount);

    // Anchor schedule to original baseline (not current ledger — prevents drift)
    record.last_charged_ledger = next_due;
    e.storage().persistent().set(&DataKey::Recurring(recurring_id), &record);
}
