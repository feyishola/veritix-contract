use crate::admin::check_admin;
use crate::balance::{decrease_supply, spend_balance};
use crate::freeze::{freeze_account, unfreeze_account};
use crate::validation::require_positive_amount;
use soroban_sdk::{symbol_short, Address, Env, Vec};

const MAX_BATCH_TARGETS: u32 = 50;

pub fn clawback_batch(e: &Env, admin: Address, targets: Vec<(Address, i128)>) {
    check_admin(e, &admin);
    if targets.len() > MAX_BATCH_TARGETS {
        panic!("batch too large");
    }
    for i in 0..targets.len() {
        let (from, amount) = targets.get(i).unwrap();
        require_positive_amount(amount);
        spend_balance(e, from.clone(), amount);
        decrease_supply(e, amount);
        e.events()
            .publish((symbol_short!("clawback"), admin.clone(), from), amount);
    }
}

pub fn freeze_batch(e: &Env, admin: Address, targets: Vec<Address>) {
    check_admin(e, &admin);
    if targets.len() > MAX_BATCH_TARGETS {
        panic!("batch too large");
    }
    for i in 0..targets.len() {
        let target = targets.get(i).unwrap();
        freeze_account(e, admin.clone(), target);
    }
    e.events()
        .publish((symbol_short!("batch_frz"), admin), targets.len());
}

pub fn unfreeze_batch(e: &Env, admin: Address, targets: Vec<Address>) {
    check_admin(e, &admin);
    if targets.len() > MAX_BATCH_TARGETS {
        panic!("batch too large");
    }
    for i in 0..targets.len() {
        let target = targets.get(i).unwrap();
        unfreeze_account(e, admin.clone(), target);
    }
    e.events()
        .publish((symbol_short!("batch_unf"), admin), targets.len());
}
