//! Validation guard module.
//! Contains reusable guards for strings, numeric ranges, ledger windows, and freeze-state checks.
//! These validators enforce consistent panic semantics across token, escrow, recurring, and split flows.

use soroban_sdk::{Address, Env, String};

pub fn require_nonempty_string(value: &String, message: &'static str) {
    if value.len() == 0 {
        panic!("{}", message);
    }
}

pub fn require_decimal_within_max(decimal: u32, max: u32) {
    if decimal > max {
        panic!("decimal exceeds maximum");
    }
}

pub fn require_positive_amount(amount: i128) {
    if amount <= 0 {
        panic!("amount must be positive");
    }
}

pub fn require_non_negative_amount(amount: i128) {
    if amount < 0 {
        panic!("amount cannot be negative");
    }
}

pub fn require_current_or_future_ledger(current_ledger: u32, expiration_ledger: u32) {
    if expiration_ledger < current_ledger {
        panic!("expiration ledger is in the past");
    }
}

pub fn require_not_frozen_account(e: &Env, addr: &Address) {
    if crate::freeze::is_frozen(e, addr) {
        panic!("account frozen");
    }
}
