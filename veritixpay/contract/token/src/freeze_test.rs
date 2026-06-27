use super::*;
use crate::balance::receive_balance;
use crate::balance::spend_balance;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_freeze_stores_true_in_persistent_storage() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    freeze_account(&env, admin, target.clone());

    assert_eq!(is_frozen(&env, &target), true);
}

#[test]
fn test_is_frozen_returns_false_for_unfrozen_address() {
    let env = Env::default();
    let target = Address::generate(&env);

    assert_eq!(is_frozen(&env, &target), false);
}

#[test]
fn test_unfreeze_removes_storage_entry() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    freeze_account(&env, admin.clone(), target.clone());
    assert_eq!(is_frozen(&env, &target), true);

    unfreeze_account(&env, admin, target.clone());
    assert_eq!(is_frozen(&env, &target), false);
}

#[test]
#[should_panic(expected = "AlreadyFrozen")]
fn test_freeze_already_frozen_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    freeze_account(&env, admin.clone(), target.clone());
    freeze_account(&env, admin, target);
}

#[test]
#[should_panic(expected = "NotFrozen")]
fn test_unfreeze_not_frozen_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    unfreeze_account(&env, admin, target);
}

#[test]
#[should_panic]
fn test_frozen_account_cannot_spend_balance() {
    let env = Env::default();
    let target = Address::generate(&env);
    let admin = Address::generate(&env);

    freeze_account(&env, admin, target.clone());
    spend_balance(&env, target, 100);
}

#[test]
fn test_frozen_account_can_receive_balance() {
    let env = Env::default();
    let target = Address::generate(&env);
    let admin = Address::generate(&env);

    freeze_account(&env, admin, target.clone());
    receive_balance(&env, target, 100);
}

#[test]
#[should_panic]
fn test_freeze_requires_admin_auth() {
    let env = Env::default();
    let target = Address::generate(&env);

    freeze_account(&env, target.clone(), target);
}