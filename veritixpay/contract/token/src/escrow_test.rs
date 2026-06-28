use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env,
};

use crate::balance::increase_supply;
use crate::balance::read_balance;
use crate::balance::read_total_supply;
use crate::contract::VeritixToken;
use crate::escrow::{
    admin_settle_escrow, create_escrow, get_escrow, refund_escrow, release_escrow, try_get_escrow,
    try_refund_escrow, try_release_escrow,
};
use crate::freeze::{freeze_account, is_frozen};
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

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        let contract_addr = e.current_contract_address();
        let before_contract_balance = read_balance(&e, contract_addr.clone());
        let before_beneficiary_balance = read_balance(&e, beneficiary.clone());

        release_escrow(&e, beneficiary.clone(), escrow_id, None);

        let record = get_escrow(&e, escrow_id);
        assert!(record.released);
        assert!(!record.refunded);

        let after_contract_balance = read_balance(&e, contract_addr);
        let after_beneficiary_balance = read_balance(&e, beneficiary.clone());

        assert_eq!(before_contract_balance - amount, after_contract_balance);
        assert_eq!(before_beneficiary_balance + amount, after_beneficiary_balance);
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

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

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
        assert_supply_matches_balances(&e, &[depositor.clone(), beneficiary.clone(), e.current_contract_address()]);

        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        assert_supply_matches_balances(&e, &[depositor.clone(), beneficiary.clone(), e.current_contract_address()]);
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id, None);
        assert_supply_matches_balances(&e, &[depositor.clone(), beneficiary.clone(), e.current_contract_address()]);
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
        assert_supply_matches_balances(&e, &[depositor.clone(), beneficiary.clone(), e.current_contract_address()]);

        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        assert_supply_matches_balances(&e, &[depositor.clone(), beneficiary.clone(), e.current_contract_address()]);
    });

    e.as_contract(&contract_id, || {
        refund_escrow(&e, depositor.clone(), escrow_id);
        assert_supply_matches_balances(&e, &[depositor.clone(), beneficiary.clone(), e.current_contract_address()]);
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
        assert_eq!(VeritixToken::escrow_count(e.clone()), 2);
    });
}

#[test]
fn test_escrow_count_getter_reflects_creations() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    e.as_contract(&contract_id, || {
        assert_eq!(VeritixToken::escrow_count(e.clone()), 0);
    });

    let depositor_one = Address::generate(&e);
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor_one.clone(), amount);
        let _ = VeritixToken::create_escrow(e.clone(), depositor_one.clone(), beneficiary.clone(), amount, 1000);
        assert_eq!(VeritixToken::escrow_count(e.clone()), 1);
    });

    let depositor_two = Address::generate(&e);
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor_two.clone(), amount);
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
            try_release_escrow(&e, beneficiary.clone(), 999, None),
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
#[should_panic(expected = "not beneficiary")]
fn test_release_unauthorized_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let hacker = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, hacker.clone(), escrow_id, None);
    });
}

#[test]
#[should_panic(expected = "not depositor")]
fn test_refund_unauthorized_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        refund_escrow(&e, beneficiary.clone(), escrow_id);
    });
}

#[test]
#[should_panic(expected = "already settled")]
fn test_double_release_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        release_escrow(&e, beneficiary.clone(), escrow_id, None);
        release_escrow(&e, beneficiary.clone(), escrow_id, None);
    });
}

#[test]
fn test_partial_release_50_percent() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;
    let release_amount = amount / 2;

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id, Some(release_amount));
        let record = get_escrow(&e, escrow_id);
        assert_eq!(record.amount, amount - release_amount);
        assert!(!record.released);
    });
}

#[test]
fn test_partial_release_to_zero_marks_as_released() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;
    let release_amount = amount / 2;

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id, Some(release_amount));
        release_escrow(&e, beneficiary.clone(), escrow_id, Some(release_amount));
        let record = get_escrow(&e, escrow_id);
        assert_eq!(record.amount, 0);
        assert!(record.released);
    });
}

#[test]
#[should_panic]
fn test_partial_release_over_remaining_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;
    let release_amount = amount / 2;

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id, Some(release_amount));
        release_escrow(&e, beneficiary.clone(), escrow_id, Some(release_amount + 1));
    });
}

#[test]
#[should_panic]
fn test_partial_release_after_full_release_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id: u32 = 0;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id, None);
        release_escrow(&e, beneficiary.clone(), escrow_id, Some(1));
    });
}

#[test]
#[should_panic(expected = "already settled")]
fn test_double_refund_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        refund_escrow(&e, depositor.clone(), escrow_id);
        refund_escrow(&e, depositor.clone(), escrow_id);
    });
}

#[test]
#[should_panic(expected = "already settled")]
fn test_release_after_refund_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        refund_escrow(&e, depositor.clone(), escrow_id);
        release_escrow(&e, beneficiary.clone(), escrow_id, None);
    });
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_create_escrow_zero_amount_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);

    e.as_contract(&contract_id, || {
        create_escrow(&e, depositor.clone(), beneficiary.clone(), 0, 1000);
    });
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_create_escrow_negative_amount_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);

    e.as_contract(&contract_id, || {
        create_escrow(&e, depositor.clone(), beneficiary.clone(), -1, 1000);
    });
}

#[test]
#[should_panic(expected = "not beneficiary")]
fn test_release_by_depositor_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, depositor.clone(), escrow_id, None);
    });
}

#[test]
#[should_panic(expected = "not depositor")]
fn test_refund_by_beneficiary_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        refund_escrow(&e, beneficiary.clone(), escrow_id);
    });
}

// --- Issue #162: Event emission tests ---

#[test]
fn test_create_escrow_emits_event() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), 1000);
        create_escrow(&e, depositor.clone(), beneficiary.clone(), 1000, 1000);
    });

    let events = e.events().all();
    assert_eq!(events.len(), 1);
    assert_eq!(events.first().unwrap().1.len(), 3);
}

#[test]
fn test_release_escrow_emits_event() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let mut escrow_id = 0u32;

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), 1000);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), 1000, 1000);
    });

    let before = e.events().all().len();

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id, None);
    });

    let events = e.events().all();
    assert_eq!(events.len(), before + 1);
    assert_eq!(events.last().unwrap().1.len(), 3);
}

#[test]
fn test_refund_escrow_emits_event() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let mut escrow_id = 0u32;

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), 1000);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), 1000, 1000);
    });

    let before = e.events().all().len();

    e.as_contract(&contract_id, || {
        refund_escrow(&e, depositor.clone(), escrow_id);
    });

    let events = e.events().all();
    assert_eq!(events.len(), before + 1);
    assert_eq!(events.last().unwrap().1.len(), 3);
}

#[test]
#[should_panic(expected = "expiration ledger is in the past")]
fn test_create_escrow_past_expiry_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let amount = 1_000i128;

    e.as_contract(&contract_id, || {
        e.ledger().with_mut(|l| l.sequence_number = 10);
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 0);
    });
}

// --- Issue #87: Frozen-account deadlock prevention tests ---

#[test]
#[should_panic(expected = "not beneficiary")]
fn test_release_blocked_when_beneficiary_frozen() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let admin = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        freeze_account(&e, admin.clone(), beneficiary.clone());
        assert!(is_frozen(&e, &beneficiary));
    });

    e.as_contract(&contract_id, || {
        release_escrow(&e, depositor.clone(), escrow_id, None);
    });
}

#[test]
fn test_expired_escrow_can_be_refunded_by_third_party() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let third_party = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 5);
    });

    e.as_contract(&contract_id, || {
        e.ledger().with_mut(|l| l.sequence_number = 6);
        let before = read_balance(&e, depositor.clone());
        refund_escrow(&e, third_party.clone(), escrow_id);
        let record = get_escrow(&e, escrow_id);
        assert!(record.refunded);
        assert_eq!(read_balance(&e, depositor.clone()), before + amount);
    });
}

#[test]
#[should_panic(expected = "not depositor")]
fn test_non_expired_escrow_cannot_be_refunded_by_third_party() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let third_party = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });

    e.as_contract(&contract_id, || {
        refund_escrow(&e, third_party.clone(), escrow_id);
    });
}

#[test]
fn test_admin_settle_escrow_when_beneficiary_frozen() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let admin = Address::generate(&e);
    let alternate = Address::generate(&e);
    let amount = 1_000i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::admin::write_admin(&e, &admin);
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        increase_supply(&e, amount);
        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        freeze_account(&e, admin.clone(), beneficiary.clone());
        assert!(is_frozen(&e, &beneficiary));

        let before = read_balance(&e, alternate.clone());
        admin_settle_escrow(&e, admin.clone(), escrow_id, alternate.clone());
        let after = read_balance(&e, alternate.clone());

        assert_eq!(after - before, amount);
        assert!(get_escrow(&e, escrow_id).released);
    });
}