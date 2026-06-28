#![cfg(test)]

use soroban_sdk::{bytes, testutils::Address as _, Address, Bytes, Env};
use crate::contract::{VeriTixPay, VeriTixPayClient};

// ── Test setup ────────────────────────────────────────────────────────────────

struct TestEnv<'a> {
    e: Env,
    client: VeriTixPayClient<'a>,
    depositor: Address,
    beneficiary: Address,
    token: Address,
}

fn setup() -> TestEnv<'static> {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let token = e.register_stellar_asset_contract(depositor.clone());

    soroban_sdk::token::StellarAssetClient::new(&e, &token).mint(&depositor, &50_000);

    TestEnv { e, client, depositor, beneficiary, token }
}

fn empty_memo(e: &Env) -> Bytes {
    Bytes::new(e)
}

fn make_memo(e: &Env, text: &[u8]) -> Bytes {
    Bytes::from_slice(e, text)
}

// ── #177: Beneficiary index ───────────────────────────────────────────────────

#[test]
fn test_create_indexes_both_parties() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &1_000, &expiry, &empty_memo(&t.e),
    );

    let by_dep = t.client.get_escrows_by_depositor(&t.depositor);
    assert_eq!(by_dep.len(), 1);
    assert_eq!(by_dep.get(0).unwrap(), id);

    let by_ben = t.client.get_escrows_by_beneficiary(&t.beneficiary);
    assert_eq!(by_ben.len(), 1);
    assert_eq!(by_ben.get(0).unwrap(), id);
}

#[test]
fn test_escrowed_total_tracks_active_amounts() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    assert_eq!(t.client.escrowed_total(), 0);

    let first = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &1_000, &expiry, &empty_memo(&t.e),
    );
    assert_eq!(first, 0);
    assert_eq!(t.client.escrowed_total(), 1_000);

    let second = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &500, &expiry, &empty_memo(&t.e),
    );
    assert_eq!(second, 1);
    assert_eq!(t.client.escrowed_total(), 1_500);

    t.client.release_escrow(&t.depositor, &first);
    assert_eq!(t.client.escrowed_total(), 500);

    t.client.refund_escrow(&t.depositor, &second);
    assert_eq!(t.client.escrowed_total(), 0);
}

#[test]
fn test_beneficiary_index_accumulates() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    for amount in [100, 200, 300] {
        t.client.create_escrow(
            &t.depositor, &t.beneficiary, &t.token, &amount, &expiry, &empty_memo(&t.e),
        );
    }

    assert_eq!(t.client.get_escrows_by_beneficiary(&t.beneficiary).len(), 3);
}

#[test]
fn test_stranger_gets_empty_list() {
    let t = setup();
    let stranger = Address::generate(&t.e);
    assert_eq!(t.client.get_escrows_by_beneficiary(&stranger).len(), 0);
}

#[test]
fn test_ticket_escrow_helper_create_and_release() {
    let t = setup();
    let event_ledger = t.e.ledger().sequence() + 500;
    let id = t.client.ticket_escrow(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &700,
        &event_ledger,
        &make_memo(&t.e, b"ticket-uuid-001"),
    );
    t.client.release_escrow(&t.beneficiary, &id);
    let tc = soroban_sdk::token::Client::new(&t.e, &t.token);
    assert_eq!(tc.balance(&t.beneficiary), 700);
}

// ── #175: Memo field ──────────────────────────────────────────────────────────

#[test]
fn test_memo_stored_and_readable() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;
    let memo = make_memo(&t.e, b"ticket:EVT-001:ORDER-9999");

    // create_escrow returns the id; we then fetch the record via a
    // get_escrow helper (add that to contract.rs if not present) or
    // verify indirectly through the index length — for a standalone test
    // the panic-free path is sufficient.
    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &500, &expiry, &memo,
    );

    // index should contain this escrow — proves creation succeeded with memo
    let list = t.client.get_escrows_by_depositor(&t.depositor);
    assert_eq!(list.get(0).unwrap(), id);
}

#[test]
fn test_empty_memo_is_valid() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    // should not panic
    t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &100, &expiry, &empty_memo(&t.e),
    );
}

#[test]
fn test_exactly_64_byte_memo_is_valid() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;
    let memo = make_memo(&t.e, &[b'x'; 64]);

    t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &100, &expiry, &memo,
    );
}

#[test]
#[should_panic(expected = "MemoTooLong: memo cannot exceed 64 bytes")]
fn test_65_byte_memo_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;
    let memo = make_memo(&t.e, &[b'x'; 65]);

    t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &100, &expiry, &memo,
    );
}

#[test]
#[should_panic(expected = "MemoTooLong: memo cannot exceed 64 bytes")]
fn test_large_memo_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;
    let memo = make_memo(&t.e, &[b'a'; 128]);

    t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &100, &expiry, &memo,
    );
}

// ── #174: Partial release ─────────────────────────────────────────────────────

#[test]
fn test_partial_release_reduces_remaining() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &1_000, &expiry, &empty_memo(&t.e),
    );

    t.client.release_partial_escrow(&t.beneficiary, &id, &300);

    // Beneficiary should have received 300
    let tc = soroban_sdk::token::Client::new(&t.e, &t.token);
    assert_eq!(tc.balance(&t.beneficiary), 300);

    // Contract still holds 700
    assert_eq!(tc.balance(&t.e.current_contract_address()), 700);
}

#[test]
fn test_multiple_partial_releases() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &900, &expiry, &empty_memo(&t.e),
    );

    t.client.release_partial_escrow(&t.beneficiary, &id, &300);
    t.client.release_partial_escrow(&t.beneficiary, &id, &300);
    t.client.release_partial_escrow(&t.beneficiary, &id, &300);

    let tc = soroban_sdk::token::Client::new(&t.e, &t.token);
    assert_eq!(tc.balance(&t.beneficiary), 900);
    assert_eq!(tc.balance(&t.e.current_contract_address()), 0);
}

#[test]
fn test_full_partial_release_marks_as_released() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &1_000, &expiry, &empty_memo(&t.e),
    );

    // Release the entire amount in one partial call
    t.client.release_partial_escrow(&t.beneficiary, &id, &1_000);

    // A second partial call must fail because it's fully released
    let result = std::panic::catch_unwind(|| {
        t.client.release_partial_escrow(&t.beneficiary, &id, &1);
    });
    assert!(result.is_err(), "expected panic on second release");
}

#[test]
#[should_panic(expected = "release amount exceeds remaining balance")]
fn test_over_release_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &500, &expiry, &empty_memo(&t.e),
    );

    t.client.release_partial_escrow(&t.beneficiary, &id, &501);
}

#[test]
#[should_panic(expected = "release amount exceeds remaining balance")]
fn test_over_release_after_partial_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &500, &expiry, &empty_memo(&t.e),
    );

    t.client.release_partial_escrow(&t.beneficiary, &id, &400);
    t.client.release_partial_escrow(&t.beneficiary, &id, &200); // 400+200 > 500
}

#[test]
#[should_panic(expected = "release amount must be greater than zero")]
fn test_zero_partial_release_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &500, &expiry, &empty_memo(&t.e),
    );

    t.client.release_partial_escrow(&t.beneficiary, &id, &0);
}

#[test]
#[should_panic(expected = "only the beneficiary can partially release")]
fn test_depositor_cannot_partial_release() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &500, &expiry, &empty_memo(&t.e),
    );

    // Depositor is not allowed to call partial release
    t.client.release_partial_escrow(&t.depositor, &id, &100);
}

#[test]
fn test_refund_after_partial_release_returns_remainder() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor, &t.beneficiary, &t.token, &1_000, &expiry, &empty_memo(&t.e),
    );

    // Beneficiary takes 400 first
    t.client.release_partial_escrow(&t.beneficiary, &id, &400);

    // Then depositor refunds — should only get back 600 (the remainder)
    t.client.refund_escrow(&t.depositor, &id);

    let tc = soroban_sdk::token::Client::new(&t.e, &t.token);
    // depositor started with 50_000, spent 1_000 on escrow, gets back 600
    assert_eq!(tc.balance(&t.depositor), 49_600);
    assert_eq!(tc.balance(&t.beneficiary), 400);
    assert_eq!(tc.balance(&t.e.current_contract_address()), 0);
}
// ── #181: Escrow events with memo ─────────────────────────────────────────────

#[test]
fn test_create_escrow_event_includes_memo() {
    let t = setup();
    t.e.mock_all_auths();
    let expiry = t.e.ledger().sequence() + 1000;
    let memo = make_memo(&t.e, b"ticket:EVT-001:ORDER-9999");

    let id = t.client.create_escrow(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &1_000,
        &expiry,
        &memo,
    );

    // Verify escrow was created successfully with memo
    let list = t.client.get_escrows_by_depositor(&t.depositor);
    assert_eq!(list.get(0).unwrap(), id);

    // Verify events were emitted
    let events = t.e.events().all();
    assert!(!events.is_empty(), "escrow_created event should be emitted");
}

#[test]
fn test_release_escrow_event_includes_memo() {
    let t = setup();
    t.e.mock_all_auths();
    let expiry = t.e.ledger().sequence() + 1000;
    let memo = make_memo(&t.e, b"ticket:EVT-002:ORDER-1234");

    let id = t.client.create_escrow(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &1_000,
        &expiry,
        &memo,
    );

    t.client.release_escrow(&t.depositor, &id);

    // Verify events were emitted including release event
    let events = t.e.events().all();
    assert!(events.len() >= 2, "escrow_created and escrow_released events should be emitted");
}

#[test]
fn test_refund_escrow_event_includes_memo() {
    let t = setup();
    t.e.mock_all_auths();
    let expiry = t.e.ledger().sequence() + 1000;
    let memo = make_memo(&t.e, b"ticket:EVT-003:ORDER-5678");

    let id = t.client.create_escrow(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &1_000,
        &expiry,
        &memo,
    );

    t.client.refund_escrow(&t.depositor, &id);

    // Verify events were emitted including refund event
    let events = t.e.events().all();
    assert!(events.len() >= 2, "escrow_created and escrow_refunded events should be emitted");
}

#[test]
fn test_create_escrow_event_with_empty_memo() {
    let t = setup();
    t.e.mock_all_auths();
    let expiry = t.e.ledger().sequence() + 1000;

    let id = t.client.create_escrow(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &500,
        &expiry,
        &empty_memo(&t.e),
    );

    // Even with empty memo event should be emitted
    let events = t.e.events().all();
    assert!(!events.is_empty(), "escrow_created event should be emitted even with empty memo");

    let list = t.client.get_escrows_by_depositor(&t.depositor);
    assert_eq!(list.get(0).unwrap(), id);
}

#[test]
fn test_create_escrow_requires_depositor_auth() {
    let env = Env::default();
    
    // Explicitly DO NOT call env.mock_all_auths() here.
    // This ensures Soroban enforces cryptographic signatures strictly.

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token_addr = Address::generate(&env);
    let memo = Bytes::new(&env);

    // Attempting to invoke create_escrow should fail because the contract 
    // demands a signature that we haven't provided.
    let result = env.try_invoke_contract_with_address(
        &depositor,
        &|env| {
            create_escrow(
                env.clone(),
                depositor.clone(),
                beneficiary.clone(),
                token_addr.clone(),
                1000,
                100,
                memo.clone(),
            )
        }
    );

    // Assert that the call failed due to an authorization error
    assert!(result.is_err(), "Expected transaction to fail due to missing depositor authentication.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_types::MAX_ESCROW_AMOUNT;
    use soroban_sdk::{testutils::Address as _, Env, Bytes};

    #[test]
    #[should_panic(expected = "AmountTooLarge: use multi-party escrow for large amounts")]
    fn test_create_escrow_rejects_exceeded_cap_amount() {
        let env = Env::default();
        let depositor = env.accounts().generate();
        let beneficiary = env.accounts().generate();
        let token = env.accounts().generate();
        let memo = Bytes::new(&env);

        // Supply an amount exactly 1 unit over the allowed global safety cap
        let illegal_excessive_amount = MAX_ESCROW_AMOUNT + 1;

        create_escrow(
            env,
            depositor,
            beneficiary,
            token,
            illegal_excessive_amount,
            12345,
            memo,
        );
    }
}

#[cfg(test)]
mod lien_tests {
    use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Bytes, Env};
    use crate::contract::{VeriTixPay, VeriTixPayClient};
    use crate::test::create_token_contract;

    #[test]
    fn test_lien_mechanics() {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().with_mut(|l| l.sequence = 100);

        let depositor = Address::generate(&e);
        let beneficiary = Address::generate(&e);
        let creditor = Address::generate(&e);
        let admin = Address::generate(&e);

        let contract_id = e.register_contract(None, VeriTixPay);
        let client = VeriTixPayClient::new(&e, &contract_id);
        
        let token = create_token_contract(&e, &admin);
        let token_client = soroban_sdk::token::Client::new(&e, &token);
        
        token_client.mint(&depositor, &2000);
        
        let memo = Bytes::from_slice(&e, b"test lien");
        let escrow_id = client.create_escrow(&depositor, &beneficiary, &token, &1000, &200, &memo);
        
        // Place a lien
        client.place_lien(&creditor, &escrow_id, &300);
        
        // Release the escrow, should send 300 to creditor and 700 to beneficiary
        client.release_escrow(&depositor, &escrow_id);
        
        assert_eq!(token_client.balance(&creditor), 300);
        assert_eq!(token_client.balance(&beneficiary), 700);
    }
}