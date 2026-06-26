use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Events as _, testutils::Ledger as _, Address, Env, String};

use crate::contract::VeritixTokenClient;

fn setup() -> (Env, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    (env, admin, user)
}

fn create_client_with_id(env: &Env) -> (Address, VeritixTokenClient<'_>) {
    let contract_id = env.register_contract(None, VeritixToken);
    (
        contract_id.clone(),
        VeritixTokenClient::new(env, &contract_id),
    )
}

fn create_client(env: &Env) -> VeritixTokenClient<'_> {
    create_client_with_id(env).1
}

fn initialize_client(client: &VeritixTokenClient<'_>, env: &Env, admin: &Address, decimal: u32) {
    client.initialize(
        admin,
        &String::from_str(env, "Veritix"),
        &String::from_str(env, "VTX"),
        &decimal,
    );
}

fn assert_supply_matches(client: &VeritixTokenClient<'_>, tracked_addresses: &[Address]) {
    let tracked_sum = tracked_addresses
        .iter()
        .fold(0i128, |sum, address| sum + client.balance(address));
    assert_eq!(client.total_supply(), tracked_sum);
}

// Verifies that initialize correctly sets admin, name, symbol, and decimals.
// If this test fails, the core token deployment flow is broken.
#[test]
fn test_initialize() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);

    assert_eq!(client.admin(), admin);
    assert_eq!(client.name(), String::from_str(&env, "Veritix"));
    assert_eq!(client.symbol(), String::from_str(&env, "VTX"));
    assert_eq!(client.decimals(), 7u32);
}

// Ensures that calling initialize twice panics — the contract must be
// single-initialize to prevent admin re-assignment after deployment.
#[test]
#[should_panic]
fn test_initialize_twice_panics() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    initialize_client(&client, &env, &admin, 7);
}

// Validates that decimal > 18 is rejected, enforcing the max precision limit.
#[test]
#[should_panic]
fn test_initialize_rejects_decimal_above_eighteen() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 19);
}

// Verifies that an empty name string is rejected during initialization.
// Prevents deployment of a token without a display name.
#[test]
#[should_panic]
fn test_initialize_rejects_empty_name() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, ""),
        &String::from_str(&env, "VTX"),
        &7u32,
    );
}

// Verifies that an empty symbol string is rejected during initialization.
#[test]
#[should_panic]
fn test_initialize_rejects_empty_symbol() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "Veritix"),
        &String::from_str(&env, ""),
        &7u32,
    );
}

// Ensures that initialize requires the admin to sign — without mock auths
// the call must panic because admin.require_auth() fails.
#[test]
#[should_panic]
fn test_initialize_requires_admin_authorization() {
    let (env, admin, _user) = setup();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
}

// Happy-path mint: admin mints 1000 tokens to a user; balance and total supply
// should both reflect the minted amount. If this fails, token issuance is broken.
#[test]
fn test_mint() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);

    assert_eq!(client.balance(&user), 1000i128);
    assert_eq!(client.total_supply(), 1000i128);
}

// Ensures that a non-admin caller cannot mint tokens — the authorization check
// must reject the call even when the caller provides their own auth.
#[test]
#[should_panic]
fn test_mint_unauthorized_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    env.set_auths(&[]);

    client.mint(&user, &user, &1000i128);
}

// Happy-path burn: user burns 500 of their 1000 tokens; balance and total
// supply decrease accordingly. Tests the basic deflationary path.
#[test]
fn test_burn() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);

    client.mint(&admin, &user, &1000i128);
    client.burn(&user, &500i128);

    assert_eq!(client.balance(&user), 500i128);
    assert_eq!(client.total_supply(), 500i128);
}

// Ensures that burning more than the user's balance panics — prevents
// negative balances and unauthorized supply reduction.
#[test]
#[should_panic]
fn test_burn_insufficient_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &100i128);

    client.burn(&user, &200i128);
}

// Happy-path transfer: sender sends 400 tokens to a receiver; balances update
// correctly and total supply stays unchanged.
#[test]
fn test_transfer() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let receiver = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.transfer(&user, &receiver, &400i128);

    assert_eq!(client.balance(&user), 600i128);
    assert_eq!(client.balance(&receiver), 400i128);
    assert_eq!(client.total_supply(), 1000i128);
}

// Ensures that transferring without sufficient balance panics — prevents
// creation of tokens out of thin air via transfer.
#[test]
#[should_panic]
fn test_transfer_insufficient_balance_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let receiver = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);

    client.transfer(&user, &receiver, &100i128);
}

// Happy-path transfer_from: spender uses allowance to transfer tokens from
// owner to a receiver; allowance is reduced by the spent amount.
#[test]
fn test_transfer_from() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &500i128, &1000u32);
    client.transfer_from(&spender, &user, &receiver, &300i128);

    assert_eq!(client.balance(&receiver), 300i128);
    assert_eq!(client.allowance(&user, &spender), 200i128);
}

// Verifies that the token_info view returns a combined struct of metadata and
// total supply in a single call, useful for frontend display.
#[test]
fn test_token_info_combines_metadata_and_supply() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &123i128);
    let info = client.token_info();
    assert_eq!(info.name, String::from_str(&env, "Veritix"));
    assert_eq!(info.symbol, String::from_str(&env, "VTX"));
    assert_eq!(info.decimal, 7);
    assert_eq!(info.total_supply, 123);
}

// Verifies that transfer_with_memo moves funds correctly and attaches a memo
// tag for off-chain correlation (e.g., ticket UUID).
#[test]
fn test_transfer_with_memo_moves_funds() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let receiver = Address::generate(&env);
    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    let memo = soroban_sdk::Bytes::from_array(&env, b"ticket-001");
    client.transfer_with_memo(&user, &receiver, &250i128, &memo);
    assert_eq!(client.balance(&user), 750);
    assert_eq!(client.balance(&receiver), 250);
}

// Tests the full approve + transfer_from cycle: owner approves spender, spender
// transfers a portion, and the allowance decreases accordingly.
#[test]
fn test_approve_and_spend_allowance() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &400i128, &1000u32);
    client.transfer_from(&spender, &user, &spender, &200i128);

    assert_eq!(client.balance(&spender), 200i128);
    assert_eq!(client.allowance(&user, &spender), 200i128);
}

// Ensures that spending the full allowance reduces it to zero, confirming
// that allowance is not stuck at a non-zero value after full consumption.
#[test]
fn test_transfer_from_spends_full_allowance_and_clears_it() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &400i128, &1000u32);
    client.transfer_from(&spender, &user, &receiver, &400i128);

    assert_eq!(client.balance(&receiver), 400i128);
    assert_eq!(client.allowance(&user, &spender), 0i128);
}

// Verifies that an allowance with expiration_ledger equal to the current ledger
// is still valid for spending — the boundary condition where expiry == current.
#[test]
fn test_allowance_expiration_equal_current_ledger_is_valid_for_current_ledger() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let current_ledger = env.ledger().sequence();

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &250i128, &current_ledger);
    client.transfer_from(&spender, &user, &receiver, &250i128);

    assert_eq!(client.balance(&receiver), 250i128);
    assert_eq!(client.allowance(&user, &spender), 0i128);
}

// Ensures that an approval with an expiration_ledger strictly in the past is
// rejected — prevents setting allowances that are already expired.
#[test]
#[should_panic]
fn test_approve_with_past_expiration_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);

    // Advance ledger so that expiration_ledger = 0 is strictly in the past.
    env.ledger().with_mut(|l| l.sequence_number = 10);
    client.approve(&user, &spender, &400i128, &0u32);
}

// Verifies that an expired allowance cannot be spent — even if it was valid
// when created, advancing the ledger past expiration makes it unusable.
#[test]
#[should_panic]
fn test_expired_allowance_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &400i128, &0u32);

    client.transfer_from(&spender, &user, &spender, &100i128);
}

// End-to-end test of admin rotation, freeze, unfreeze, and their storage
// representations — verifies that the views reflect on-chain state changes.
#[test]
fn test_admin_and_freeze_views_follow_state_changes() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let (contract_id, client) = create_client_with_id(&env);
    let new_admin = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);

    assert_eq!(client.admin(), admin);
    assert!(!client.is_frozen(&user));
    assert_eq!(
        env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get::<crate::storage_types::DataKey, bool>(&crate::storage_types::DataKey::Freeze(
                    user.clone(),
                ))
        }),
        None
    );

    client.set_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);

    client.freeze(&user);
    assert!(client.is_frozen(&user));
    assert_eq!(
        env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get::<crate::storage_types::DataKey, bool>(&crate::storage_types::DataKey::Freeze(
                    user.clone(),
                ))
        }),
        Some(true)
    );

    client.unfreeze(&user);
    assert!(!client.is_frozen(&user));
    assert_eq!(
        env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get::<crate::storage_types::DataKey, bool>(&crate::storage_types::DataKey::Freeze(
                    user.clone(),
                ))
        }),
        None
    );
}

// Verifies that clawback reduces the target's balance and total supply by the
// clawed-back amount — confirms deflationary admin recovery works.
#[test]
fn test_clawback_reduces_total_supply() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);

    client.mint(&admin, &user, &1000i128);
    assert_eq!(client.balance(&user), 1000i128);
    assert_eq!(client.total_supply(), 1000i128);

    client.clawback(&admin, &user, &300i128);

    assert_eq!(client.balance(&user), 700i128);
    assert_eq!(client.total_supply(), 700i128);
}

// Smoke test covering the full admin lifecycle: set_admin, freeze, unfreeze,
// and clawback in sequence — validates that all admin operations work together.
#[test]
fn test_admin_freeze_and_clawback_smoke_via_client() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let new_admin = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1_000i128);

    client.set_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);

    client.freeze(&user);
    assert!(client.is_frozen(&user));

    client.unfreeze(&user);
    assert!(!client.is_frozen(&user));

    client.clawback(&new_admin, &user, &250i128);
    assert_eq!(client.balance(&user), 750i128);
    assert_eq!(client.total_supply(), 750i128);
}

// Critical invariant: after each core token operation (mint, transfer, burn,
// clawback), the sum of all tracked balances must equal total_supply.
// If this fails, tokens are being created or destroyed silently.
#[test]
fn test_core_token_operations_preserve_supply_invariant() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let receiver = Address::generate(&env);
    let tracked = [user.clone(), receiver.clone()];

    initialize_client(&client, &env, &admin, 7);

    client.mint(&admin, &user, &1_000i128);
    assert_supply_matches(&client, &tracked);

    client.transfer(&user, &receiver, &250i128);
    assert_supply_matches(&client, &tracked);

    client.burn(&receiver, &100i128);
    assert_supply_matches(&client, &tracked);

    client.clawback(&admin, &user, &150i128);
    assert_supply_matches(&client, &tracked);
}

// Ensures that a non-admin caller cannot invoke clawback — authorization is
// enforced even if the caller has sufficient balance.
#[test]
#[should_panic]
fn test_clawback_unauthorized_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    env.set_auths(&[]);

    client.clawback(&user, &user, &300i128);
}

// Verifies that a frozen account can still receive tokens via escrow release.
// Freeze prevents sending but not receiving — this prevents deadlock where
// escrowed funds become stuck forever.
#[test]
fn test_frozen_account_can_receive_from_escrow_release() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let beneficiary = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1_000i128);

    // Freeze the beneficiary — they should still be able to receive escrowed funds.
    client.freeze(&beneficiary);
    assert!(client.is_frozen(&beneficiary));

    let escrow_id = client.create_escrow(&user, &beneficiary, &1_000i128, &1000u32);

    // Release escrow to the frozen beneficiary — must not panic.
    client.release_escrow(&user, &escrow_id);

    assert_eq!(client.balance(&beneficiary), 1_000i128);
}

// Verifies that freeze and unfreeze emit the correct events with proper topic
// structure and payload, enabling off-chain indexers to track freeze state.
#[test]
fn test_freeze_and_unfreeze_emit_observable_events() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    let before_freeze = env.events().all().len();

    client.freeze(&user);
    let freeze_events = env.events().all();
    assert_eq!(freeze_events.len(), before_freeze + 1);
    let freeze_event = freeze_events.last().unwrap();
    assert_eq!(freeze_event.1.len(), 2);
    // Topics: (frozen_symbol, user), data: admin
    assert!(!freeze_event.1.is_empty());

    let before_unfreeze = env.events().all().len();

    client.unfreeze(&user);
    let unfreeze_events = env.events().all();
    assert_eq!(unfreeze_events.len(), before_unfreeze + 1);
    let unfreeze_event = unfreeze_events.last().unwrap();
    assert_eq!(unfreeze_event.1.len(), 2);
    // Topics: (unfrozen_symbol, user), data: admin
    assert!(!unfreeze_event.1.is_empty());
}

// Ensures that only the admin can freeze accounts — a non-admin caller must
// be rejected even with mock auths cleared.
#[test]
#[should_panic]
fn test_freeze_by_non_admin_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let attacker = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    env.set_auths(&[]);

    client.freeze(&attacker);
    let _ = user; // suppress unused warning
}

// Ensures that a spender cannot transfer_from more than the approved allowance
// — the allowance cap is enforced even if the owner's balance is sufficient.
#[test]
#[should_panic]
fn test_transfer_from_over_allowance_panics() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1_000i128);
    client.approve(&user, &spender, &400i128, &1_000u32);

    // Confirm the allowance key exists before spending.
    let key = crate::storage_types::DataKey::Allowance(crate::storage_types::AllowanceDataKey {
        from: user.clone(),
        spender: spender.clone(),
    });
    let before = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get::<crate::storage_types::DataKey, crate::storage_types::AllowanceValue>(&key)
    });
    assert!(before.is_some(), "expected Allowance key to exist after approve");

    // Spend the entire allowance in one transfer_from call.
    client.transfer_from(&spender, &user, &receiver, &400i128);
    assert_eq!(client.allowance(&user, &spender), 0i128);

    // The underlying storage key must be gone, not stored as amount=0.
    let after = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get::<crate::storage_types::DataKey, crate::storage_types::AllowanceValue>(&key)
    });
    assert_eq!(
        after, None,
        "expected Allowance storage key to be removed after full spend, not stored as zero"
    );
}
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &100i128, &1000u32);

    // Attempt to spend more than the approved allowance.
    client.transfer_from(&spender, &user, &receiver, &200i128);
}

// --- Issue #162: Event emission tests ---

// NOTE: freeze/unfreeze currently emit no events — this is a known gap.
// The functions below test the events that ARE emitted.

// Verifies that unfreeze_account removes the Freeze(addr) storage key entirely,
// not merely writing `false`. This ensures storage is cleaned up and avoids
// stale keys occupying ledger rent indefinitely.
#[test]
fn test_unfreeze_removes_storage_key() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let (contract_id, client) = create_client_with_id(&env);

    initialize_client(&client, &env, &admin, 7);

    // Freeze the user — the Freeze(user) key should now exist as true.
    client.freeze(&user);
    assert!(client.is_frozen(&user));

    // Verify the raw storage key is present before unfreeze.
    let key_before = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get::<crate::storage_types::DataKey, bool>(
                &crate::storage_types::DataKey::Freeze(user.clone()),
            )
    });
    assert_eq!(key_before, Some(true), "expected Freeze key to exist after freeze");

    // Unfreeze — the key must be removed, not set to false.
    client.unfreeze(&user);
    assert!(!client.is_frozen(&user));

    let key_after = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get::<crate::storage_types::DataKey, bool>(
                &crate::storage_types::DataKey::Freeze(user.clone()),
            )
    });
    assert_eq!(
        key_after, None,
        "expected Freeze storage key to be fully removed after unfreeze, not set to false"
    );
}

// Verifies that set_admin emits a single event with (admin_set, old_admin)
// topic and new_admin as data.
#[test]
fn test_set_admin_emits_event() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let new_admin = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    let before = env.events().all().len();

    client.set_admin(&new_admin);

    let events = env.events().all();
    assert_eq!(events.len(), before + 1);
    // Topics: (admin_set, current_admin), data: new_admin
    assert_eq!(events.last().unwrap().1.len(), 2);
}
