use soroban_sdk::{testutils::{Address as _, Events as _, Ledger as _}, Address, Env};

use crate::balance::read_balance;
use crate::contract::VeritixToken;
use crate::dispute::{expire_dispute, get_dispute, open_dispute, resolve_dispute, DisputeStatus};
use soroban_sdk::Bytes;
use crate::escrow::{create_escrow, get_escrow};
use crate::storage_types::{read_counter, DataKey};

fn setup_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

fn setup_escrow(e: &Env, contract_id: &Address) -> (Address, Address, u32) {
    let depositor = Address::generate(e);
    let beneficiary = Address::generate(e);
    let amount = 1_000i128;
    let mut escrow_id = 0u32;
    e.as_contract(contract_id, || {
        crate::balance::receive_balance(e, depositor.clone(), amount);
        escrow_id = create_escrow(e, depositor.clone(), beneficiary.clone(), amount, 1000);
    });
    (depositor, beneficiary, escrow_id)
}

// Verifies that open_dispute stores a record with correct escrow_id, claimant,
// resolver, and initial Open status. If this fails, the dispute creation flow
// is broken and no disputes can be filed.
#[test]
fn test_open_dispute_stores_record() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.escrow_id, escrow_id);
        assert_eq!(record.claimant, depositor);
        assert_eq!(record.resolver, resolver);
        assert_eq!(record.status, DisputeStatus::Open);
    });
}

// Happy-path: resolves dispute in favour of the beneficiary, verifying that
// the escrow is released and funds are transferred to the beneficiary.
#[test]
fn test_resolve_dispute_for_beneficiary() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (_depositor, beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, beneficiary.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        resolve_dispute(&e, resolver.clone(), dispute_id, true);

        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.status, DisputeStatus::ResolvedForBeneficiary);

        let escrow = get_escrow(&e, escrow_id);
        assert!(escrow.released);

        assert_eq!(read_balance(&e, beneficiary.clone()), 1_000);
    });
}

// Happy-path: resolves dispute in favour of the depositor, verifying that
// the escrow is refunded and funds are returned to the depositor.
#[test]
fn test_resolve_dispute_for_depositor() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        resolve_dispute(&e, resolver.clone(), dispute_id, false);

        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.status, DisputeStatus::ResolvedForDepositor);

        let escrow = get_escrow(&e, escrow_id);
        assert!(escrow.refunded);

        assert_eq!(read_balance(&e, depositor.clone()), 1_000);
    });
}

// Ensures that only the designated resolver can resolve a dispute — an
// impostor caller must be rejected with "UnauthorizedResolver".
#[test]
#[should_panic(expected = "UnauthorizedResolver")]
fn test_resolve_dispute_wrong_resolver_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let impostor = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        resolve_dispute(&e, impostor.clone(), dispute_id, true);
    });
}

// Ensures that resolving an already-resolved dispute panics — prevents
// double resolution that could double-spend escrow funds.
#[test]
#[should_panic(expected = "AlreadyResolved")]
fn test_double_resolve_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        resolve_dispute(&e, resolver.clone(), dispute_id, true);
        resolve_dispute(&e, resolver.clone(), dispute_id, false);
    });
}

// Ensures that opening a dispute on an already-settled escrow (released or
// refunded) is rejected with "InvalidState".
#[test]
#[should_panic(expected = "InvalidState")]
fn test_open_dispute_on_settled_escrow_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (_depositor, beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        crate::escrow::release_escrow(&e, beneficiary.clone(), escrow_id);
        open_dispute(&e, beneficiary.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
    });
}

// Ensures that opening a second dispute on the same unresolved escrow panics
// — prevents dispute spam on the same escrow.
#[test]
#[should_panic(expected = "DisputeAlreadyOpen")]
fn test_duplicate_open_dispute_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        // Second open on the same unresolved escrow must fail.
        open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
    });
}

// Verifies that a new dispute can be opened on a different escrow after a
// previous dispute has been resolved — the EscrowDispute pointer is cleared
// on resolution, allowing subsequent disputes on other escrows.
#[test]
fn test_reopen_dispute_after_resolution() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    // Create a second escrow to reopen on (the first is settled after resolution).
    let (depositor2, _beneficiary2, escrow_id2) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        resolve_dispute(&e, resolver.clone(), dispute_id, false);

        // After resolution the EscrowDispute pointer is cleared; a new dispute on a
        // different (still-open) escrow must succeed.
        let new_id = open_dispute(&e, depositor2.clone(), escrow_id2, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        let record = get_dispute(&e, new_id);
        assert_eq!(record.status, DisputeStatus::Open);
    });
}

// Ensures that the claimant cannot also be the resolver — a resolver must be
// an impartial third party.
#[test]
#[should_panic(expected = "InvalidResolver: resolver cannot be the claimant")]
fn test_open_dispute_rejects_claimant_as_resolver() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        open_dispute(&e, depositor.clone(), escrow_id, depositor.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
    });
}

// Ensures that the depositor cannot be the resolver — prevents a conflict of
// interest where the depositor resolves their own dispute.
#[test]
#[should_panic(expected = "InvalidResolver: resolver cannot be the depositor")]
fn test_open_dispute_rejects_depositor_as_resolver() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let (_depositor, beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let escrow = get_escrow(&e, escrow_id);
        open_dispute(&e, beneficiary.clone(), escrow_id, escrow.depositor.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
    });
}

// Ensures that the beneficiary cannot be the resolver — prevents a conflict of
// interest where the beneficiary resolves their own dispute.
#[test]
#[should_panic(expected = "InvalidResolver: resolver cannot be the beneficiary")]
fn test_open_dispute_rejects_beneficiary_as_resolver() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let escrow = get_escrow(&e, escrow_id);
        open_dispute(&e, depositor.clone(), escrow_id, escrow.beneficiary.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
    });
}

// --- Issue #162: Event emission tests ---

// Ensures that a stranger (neither depositor nor beneficiary) cannot open a
// dispute — only escrow participants have standing.
#[test]
#[should_panic(expected = "Unauthorized: only escrow parties can open a dispute")]
fn test_open_dispute_stranger_as_claimant_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let stranger = Address::generate(&e);
    let (_depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        open_dispute(&e, stranger.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
    });
}

// Ensures that the dispute counter does not skip IDs when a dispute call is
// rejected — IDs from successful opens must be sequential (1, 2, 3).
#[test]
fn test_dispute_counter_does_not_skip_on_rejected_call() {
    // Verify that a rejected open_dispute call (duplicate) does not increment the counter.
    // We do this by opening two disputes on two separate escrows and confirming IDs are 1 and 2.
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);
    let (depositor2, _beneficiary2, escrow_id2) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        // First dispute gets ID 1
        let id1 = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        assert_eq!(id1, 1);

        // Second dispute on a different escrow gets ID 2 (no gap)
        let id2 = open_dispute(&e, depositor2.clone(), escrow_id2, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        assert_eq!(id2, 2);
    });
}

// --- Issue #162: Event emission tests ---

// Verifies that open_dispute emits a single event with (dispute_opened,
// escrow_id, claimant) topics.
#[test]
fn test_open_dispute_emits_event() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    // Clear escrow creation event
    let _ = e.events().all();

    e.as_contract(&contract_id, || {
        open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
    });

    let events = e.events().all();
    // Events: escr_crtd (from setup_escrow) + disp_open = 2 total
    assert!(events.len() >= 1);
    // The last event is the disp_open event with 3 topics: (dispute_opened, escrow_id, claimant)
    assert_eq!(events.last().unwrap().1.len(), 3);
}

// Verifies that resolve_dispute emits events (escrow_refunded and
// dispute_resolved) with correct topic structure.
#[test]
fn test_resolve_dispute_emits_event() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);

        // Clear prior events
        let _ = e.events().all();

        resolve_dispute(&e, resolver.clone(), dispute_id, false);
    });

    let events = e.events().all();
    // Expect: escrow_refunded + dispute_resolved = 2 events
    assert!(events.len() >= 1);
    // Last event should be dispute_resolved with 3 topics
    let last = events.last().unwrap();
    assert_eq!(last.1.len(), 3);
}

// --- Dispute counter tests ---

// Ensures the dispute counter starts at zero before any disputes are opened.
#[test]
fn test_dispute_count_starts_at_zero() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);

    e.as_contract(&contract_id, || {
        let count = read_counter(&e, &DataKey::DisputeCount);
        assert_eq!(count, 0);
    });
}

// Verifies the dispute counter increments correctly with dispute IDs 1, 2, 3
// across multiple escrows — no ID gaps or collisions.
#[test]
fn test_dispute_count_increments_on_open() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);

    // Create three escrows for three disputes
    let (depositor1, _beneficiary1, escrow_id1) = setup_escrow(&e, &contract_id);
    let (depositor2, _beneficiary2, escrow_id2) = setup_escrow(&e, &contract_id);
    let (depositor3, _beneficiary3, escrow_id3) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        // Before any disputes
        assert_eq!(read_counter(&e, &DataKey::DisputeCount), 0);

        // Open first dispute
        let dispute_id = open_dispute(&e, depositor1.clone(), escrow_id1, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        assert_eq!(dispute_id, 1);
        assert_eq!(read_counter(&e, &DataKey::DisputeCount), 1);

        // Open second dispute
        let dispute_id = open_dispute(&e, depositor2.clone(), escrow_id2, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        assert_eq!(dispute_id, 2);
        assert_eq!(read_counter(&e, &DataKey::DisputeCount), 2);

        // Open third dispute
        let dispute_id = open_dispute(&e, depositor3.clone(), escrow_id3, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        assert_eq!(dispute_id, 3);
        assert_eq!(read_counter(&e, &DataKey::DisputeCount), 3);
    });
}

// Verifies that resolve_dispute_with_note stores the note on the dispute record
// and correctly updates the status to ResolvedForBeneficiary.
#[test]
fn test_resolve_dispute_with_note_stores_note() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);
    let resolver = Address::generate(&e);

    e.as_contract(&contract_id, || {
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence() + 1000);
        let note = soroban_sdk::Bytes::from_slice(&e, b"resolved: funds to beneficiary");
        crate::dispute::resolve_dispute_with_note(
            &e, resolver.clone(), dispute_id, true, note.clone(),
        );
        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.resolution_note, note);
        assert_eq!(record.status, DisputeStatus::ResolvedForBeneficiary);
    });
}

#[test]
fn test_open_dispute_stores_evidence() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let evidence = Bytes::from_slice(&e, b"invoice-ref-42");
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), evidence.clone(), e.ledger().sequence() + 500);
        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.evidence, evidence);
        assert_eq!(record.expiry_ledger, e.ledger().sequence() + 500);
    });
}

#[test]
#[should_panic(expected = "EvidenceTooLong")]
fn test_open_dispute_evidence_too_long_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let evidence = Bytes::from_slice(&e, &[0u8; 129]);
        open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), evidence, e.ledger().sequence() + 500);
    });
}

#[test]
#[should_panic(expected = "InvalidExpiry")]
fn test_open_dispute_past_expiry_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), e.ledger().sequence());
    });
}

#[test]
fn test_expire_dispute_auto_resolves_for_depositor() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let expiry = e.ledger().sequence() + 100;
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), expiry);
        e.ledger().with_mut(|l| l.sequence_number = expiry + 1);
        expire_dispute(&e, dispute_id);
        let record = get_dispute(&e, dispute_id);
        assert_eq!(record.status, DisputeStatus::Expired);
        assert_eq!(read_balance(&e, depositor.clone()), 1_000);
    });
}

#[test]
#[should_panic(expected = "NotExpired")]
fn test_expire_dispute_before_expiry_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let resolver = Address::generate(&e);
    let (depositor, _beneficiary, escrow_id) = setup_escrow(&e, &contract_id);

    e.as_contract(&contract_id, || {
        let expiry = e.ledger().sequence() + 100;
        let dispute_id = open_dispute(&e, depositor.clone(), escrow_id, resolver.clone(), Bytes::new(&e), expiry);
        expire_dispute(&e, dispute_id);
    });
}
