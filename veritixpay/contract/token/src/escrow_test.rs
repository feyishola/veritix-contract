use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env,
};

use crate::balance::increase_supply;
use crate::balance::read_balance;
use crate::balance::read_total_supply;
use crate::contract::VeritixToken;
use crate::escrow::{
    create_escrow, get_escrow, refund_escrow, release_escrow, try_get_escrow, try_refund_escrow,
    try_release_escrow,
};
use crate::storage_types::{read_counter, DataKey};

// Helper to create a fresh Env with mock auth enabled.
fn setup_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

fn assert_supply_matches_balances(e: &Env, addresses: &[Address]) {
    let tracked_sum = addresses
        .iter()
        .fold(0i128, |sum, address| sum + read_balance(e, address.clone()));
    assert_eq!(read_total_supply(e), tracked_sum);
}

#[test]
fn test_create_escrow_stores_record() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        // Pre-fund depositor so spend_balance in create_escrow succeeds.
        crate::balance::receive_balance(&e, depositor.clone(), amount);

        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        let record = get_escrow(&e, escrow_id);

        assert_eq!(record.id, escrow_id);
        assert_eq!(record.depositor, depositor);
        assert_eq!(record.beneficiary, beneficiary);
        assert_eq!(record.amount, amount);
        assert!(!record.released);
        assert!(!record.refunded);
    });

    assert_eq!(e.events().all().len(), 1);
}

#[test]
fn test_release_escrow_happy_path() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    // First call: create the escrow in its own contract frame.
    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    // Second call: release the escrow and check balances.
    e.as_contract(&contract_id, || {
        // Capture balances before
        let contract_addr = e.current_contract_address();
        let before_contract_balance = read_balance(&e, contract_addr.clone());
        let before_beneficiary_balance = read_balance(&e, beneficiary.clone());

        release_escrow(&e, beneficiary.clone(), escrow_id);

        let record = get_escrow(&e, escrow_id);
        assert!(record.released);
        assert!(!record.refunded);

        // After release: contract should lose amount, beneficiary gains amount.
        let after_contract_balance = read_balance(&e, contract_addr);
        let after_beneficiary_balance = read_balance(&e, beneficiary.clone());

        assert_eq!(before_contract_balance - amount, after_contract_balance);
        assert_eq!(
            before_beneficiary_balance + amount,
            after_beneficiary_balance
        );
    });

    assert_eq!(e.events().all().len(), 2);
}

#[test]
fn test_refund_escrow_happy_path() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    // First call: create the escrow in its own contract frame.
    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    // Second call: refund the escrow and check balances.
    e.as_contract(&contract_id, || {
        let contract_addr = e.current_contract_address();
        let before_contract_balance = read_balance(&e, contract_addr.clone());
        let before_depositor_balance = read_balance(&e, depositor.clone());

        refund_escrow(&e, depositor.clone(), escrow_id);

        let record = get_escrow(&e, escrow_id);
        assert!(record.refunded);
        assert!(!record.released);

        let after_contract_balance = read_balance(&e, contract_addr);
        let after_depositor_balance = read_balance(&e, depositor.clone());

        assert_eq!(before_contract_balance - amount, after_contract_balance);
        assert_eq!(before_depositor_balance + amount, after_depositor_balance);
    });

    assert_eq!(e.events().all().len(), 2);
}

#[test]
fn test_escrow_create_and_release_preserve_supply_invariant() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        increase_supply(&e, amount);
        assert_supply_matches_balances(
            &e,
            &[
                depositor.clone(),
                beneficiary.clone(),
                e.current_contract_address(),
            ],
        );

        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        assert_supply_matches_balances(
            &e,
            &[
                depositor.clone(),
                beneficiary.clone(),
                e.current_contract_address(),
            ],
        );
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id);
        assert_supply_matches_balances(
            &e,
            &[
                depositor.clone(),
                beneficiary.clone(),
                e.current_contract_address(),
            ],
        );
    });
}

#[test]
fn test_escrow_create_and_refund_preserve_supply_invariant() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        increase_supply(&e, amount);
        assert_supply_matches_balances(
            &e,
            &[
                depositor.clone(),
                beneficiary.clone(),
                e.current_contract_address(),
            ],
        );

        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        assert_supply_matches_balances(
            &e,
            &[
                depositor.clone(),
                beneficiary.clone(),
                e.current_contract_address(),
            ],
        );
    });

    e.as_contract(&contract_id, || {
        refund_escrow(&e, depositor.clone(), escrow_id);
        assert_supply_matches_balances(
            &e,
            &[
                depositor.clone(),
                beneficiary.clone(),
                e.current_contract_address(),
            ],
        );
    });
}

#[test]
fn test_create_escrow_increments_counter() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let depositor_one = Address::generate(&e);
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor_one.clone(), amount);
        let escrow_id = create_escrow(&e, depositor_one.clone(), beneficiary.clone(), amount, 1000);
        assert_eq!(escrow_id, 1);
    });

    let depositor_two = Address::generate(&e);
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor_two.clone(), amount);
        let escrow_id = create_escrow(&e, depositor_two.clone(), beneficiary.clone(), amount, 1000);
        assert_eq!(escrow_id, 2);
    });

    e.as_contract(&contract_id, || {
        assert_eq!(read_counter(&e, &DataKey::EscrowCount), 2);
        // Also test the public getter
        assert_eq!(VeritixToken::escrow_count(e.clone()), 2);
    });
}

#[test]
fn test_escrow_count_getter_reflects_creations() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    // Initially zero
    e.as_contract(&contract_id, || {
        assert_eq!(VeritixToken::escrow_count(e.clone()), 0);
    });

    // Create one escrow
    let depositor_one = Address::generate(&e);
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor_one.clone(), amount);
        let _ = VeritixToken::create_escrow(
            e.clone(),
            depositor_one.clone(),
            beneficiary.clone(),
            amount,
        );
        let _ = VeritixToken::create_escrow(e.clone(), depositor_one.clone(), beneficiary.clone(), amount, 1000);
        assert_eq!(VeritixToken::escrow_count(e.clone()), 1);
    });

    // Create another escrow
    let depositor_two = Address::generate(&e);
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor_two.clone(), amount);
        let _ = VeritixToken::create_escrow(
            e.clone(),
            depositor_two.clone(),
            beneficiary.clone(),
            amount,
        );
        let _ = VeritixToken::create_escrow(e.clone(), depositor_two.clone(), beneficiary.clone(), amount, 1000);
        assert_eq!(VeritixToken::escrow_count(e.clone()), 2);
    });
}

#[test]
fn test_get_escrow_missing_id_returns_not_found_error() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);

    e.as_contract(&contract_id, || {
        assert_eq!(try_get_escrow(&e, 999), Err("escrow not found"));
    });
}

#[test]
fn test_release_missing_id_returns_not_found_error() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let beneficiary = Address::generate(&e);

    e.as_contract(&contract_id, || {
        assert_eq!(
            try_release_escrow(&e, beneficiary.clone(), 999),
            Err("escrow not found")
        );
    });
}

#[test]
fn test_refund_missing_id_returns_not_found_error() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);

    e.as_contract(&contract_id, || {
        assert_eq!(
            try_refund_escrow(&e, depositor.clone(), 999),
            Err("escrow not found")
        );
    });
}

#[test]
#[should_panic(expected = "InvalidEscrow: depositor and beneficiary cannot be the same address")]
fn test_create_escrow_same_address_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let addr = Address::generate(&e);
    let amount = 1_000i128;

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, addr.clone(), amount);
        create_escrow(&e, addr.clone(), addr.clone(), amount, 1000);
    });
}

#[test]
#[should_panic]
fn test_release_unauthorized_panics() {
    let e = setup_env();
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let hacker = Address::generate(&e);
    let amount = 1_000i128;

    crate::balance::receive_balance(&e, depositor.clone(), amount);

    let escrow_id = create_escrow(&e, depositor, beneficiary, amount, 1000);

    // Hacker (not beneficiary) tries to release.
    release_escrow(&e, hacker, escrow_id);
}

#[test]
#[should_panic]
fn test_refund_unauthorized_panics() {
    let e = setup_env();
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    crate::balance::receive_balance(&e, depositor.clone(), amount);

    let escrow_id = create_escrow(&e, depositor, beneficiary.clone(), amount, 1000);

    // Beneficiary (not depositor) tries to refund.
    refund_escrow(&e, beneficiary, escrow_id);
}

#[test]
#[should_panic]
fn test_double_release_panics() {
    let e = setup_env();
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    crate::balance::receive_balance(&e, depositor.clone(), amount);

    let escrow_id = create_escrow(&e, depositor, beneficiary.clone(), amount, 1000);

    release_escrow(&e, beneficiary.clone(), escrow_id);
    // Second release should panic.
    release_escrow(&e, beneficiary, escrow_id);
}

#[test]
#[should_panic]
fn test_double_refund_panics() {
    let e = setup_env();
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    crate::balance::receive_balance(&e, depositor.clone(), amount);

    let escrow_id = create_escrow(&e, depositor.clone(), beneficiary, amount, 1000);

    refund_escrow(&e, depositor.clone(), escrow_id);
    // Second refund should panic.
    refund_escrow(&e, depositor, escrow_id);
}

#[test]
#[should_panic]
fn test_release_after_refund_panics() {
    let e = setup_env();
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    crate::balance::receive_balance(&e, depositor.clone(), amount);

    let escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);

    refund_escrow(&e, depositor, escrow_id);
    // Any attempt to release after refund should panic.
    release_escrow(&e, beneficiary, escrow_id);
}
