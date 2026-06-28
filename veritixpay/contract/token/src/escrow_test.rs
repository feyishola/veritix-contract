use crate::test::{setup_env, VeritixToken};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_escrow(e: &Env, depositor: Address, beneficiary: Address, amount: i128, expiry: u32) -> u32 {
    e.as_contract(&e.current_contract_address(), || {
        crate::escrow::create_escrow(e, depositor, beneficiary, amount, expiry)
    })
}

fn release_escrow(e: &Env, caller: Address, escrow_id: u32) {
    e.as_contract(&e.current_contract_address(), || {
        crate::escrow::release_escrow(e, caller, escrow_id)
    })
}

fn refund_escrow(e: &Env, caller: Address, escrow_id: u32) {
    e.as_contract(&e.current_contract_address(), || {
        crate::escrow::refund_escrow(e, caller, escrow_id)
    })
}

fn admin_settle_escrow(e: &Env, admin: Address, escrow_id: u32, recipient: Address) {
    e.as_contract(&e.current_contract_address(), || {
        crate::escrow::admin_settle_escrow(e, admin, escrow_id, recipient)
    })
}

fn get_escrow(e: &Env, escrow_id: u32) -> crate::escrow::EscrowRecord {
    e.as_contract(&e.current_contract_address(), || {
        crate::escrow::get_escrow(e, escrow_id)
    })
}

fn freeze_account(e: &Env, admin: Address, id: Address) {
    e.as_contract(&e.current_contract_address(), || {
        crate::admin::freeze_account(e, admin, id)
    })
}

fn is_frozen(e: &Env, id: &Address) -> bool {
    e.as_contract(&e.current_contract_address(), || {
        crate::admin::is_frozen(e, id)
    })
}

fn read_balance(e: &Env, id: Address) -> i128 {
    e.as_contract(&e.current_contract_address(), || {
        crate::balance::read_balance(e, id)
    })
}

fn increase_supply(e: &Env, amount: i128) {
    e.as_contract(&e.current_contract_address(), || {
        let admin = crate::admin::read_admin(e);
        crate::admin::increase_supply(e, admin, amount)
    })
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
    assert_eq!(events.first().unwrap().0.len(), 3);
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

    let _ = e.events().all();

    e.as_contract(&contract_id, || {
        release_escrow(&e, beneficiary.clone(), escrow_id);
    });

    let events = e.events().all();
    assert_eq!(events.len(), 1);
    assert_eq!(events.first().unwrap().0.len(), 3);
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

    let _ = e.events().all();

    e.as_contract(&contract_id, || {
        refund_escrow(&e, depositor.clone(), escrow_id);
    });

    let events = e.events().all();
    assert_eq!(events.len(), 1);
    assert_eq!(events.first().unwrap().0.len(), 3);
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
        e.ledger().set_sequence_number(10);
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
        release_escrow(&e, depositor.clone(), escrow_id);
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
        e.ledger().set_sequence_number(6);
        let before = read_balance(&e, depositor.clone());
        refund_escrow(&e, third_party.clone(), escrow_id);
        let record = get_escrow(&e, escrow_id);
        assert!(record.refunded);
        assert_eq!(read_balance(&e, depositor.clone()), before + amount);
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
fn test_admin_settle_escrow_when_depositor_frozen() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let admin = Address::generate(&e);
    let amount = 500i128;

    let mut escrow_id = 0u32;
    e.as_contract(&contract_id, || {
        crate::admin::write_admin(&e, &admin);
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        increase_supply(&e, amount);

        escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        freeze_account(&e, admin.clone(), depositor.clone());
        admin_settle_escrow(&e, admin.clone(), escrow_id, beneficiary.clone());

        assert_eq!(read_balance(&e, beneficiary.clone()), amount);
        assert!(get_escrow(&e, escrow_id).released);
    });
}

#[test]
#[should_panic(expected = "already settled")]
fn test_admin_settle_already_settled_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let admin = Address::generate(&e);
    let amount = 1_000i128;

    e.as_contract(&contract_id, || {
        crate::admin::write_admin(&e, &admin);
        crate::balance::receive_balance(&e, depositor.clone(), amount);
        let escrow_id = create_escrow(&e, depositor.clone(), beneficiary.clone(), amount, 1000);
        release_escrow(&e, beneficiary.clone(), escrow_id);
        admin_settle_escrow(&e, admin.clone(), escrow_id, beneficiary.clone());
    });
}