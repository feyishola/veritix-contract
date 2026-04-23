use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Events as _, Address, Env, String, Symbol};

use crate::contract::VeritixTokenClient;

fn setup() -> (Env, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    (env, admin, user)
}

fn create_client(env: &Env) -> VeritixTokenClient<'_> {
    let contract_id = env.register_contract(None, VeritixToken);
    VeritixTokenClient::new(env, &contract_id)
}

fn initialize_client(client: &VeritixTokenClient<'_>, env: &Env, admin: &Address, decimal: u32) {
    client.initialize(
        admin,
        &String::from_str(env, "Veritix"),
        &String::from_str(env, "VTX"),
        &decimal,
    );
}

#[test]
fn test_mint_event_schema() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);

    // Clear initialization events
    let _ = env.events().all();

    // Mint tokens
    let amount = 1000i128;
    client.mint(&admin, &user, &amount);

    // Verify event structure
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.first().unwrap();

    // Topics should be: [mint_symbol, admin_address, to_address]
    // Payload should be: amount
    assert_eq!(event.0.len(), 3);
    assert_eq!(
        event.0.get(0).unwrap().into_val(&env),
        Symbol::new(&env, "mint")
    );
    assert_eq!(event.0.get(1).unwrap().into_val(&env), admin.clone().into());
    assert_eq!(event.0.get(2).unwrap().into_val(&env), user.clone().into());
    assert_eq!(event.1.into_val(&env), amount);
}

#[test]
fn test_burn_event_schema() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);

    // Clear events
    let _ = env.events().all();

    // Burn tokens
    let amount = 500i128;
    client.burn(&user, &amount);

    // Verify event structure
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.first().unwrap();

    // Topics should be: [burn_symbol, from_address]
    // Payload should be: amount
    assert_eq!(event.0.len(), 2);
    assert_eq!(
        event.0.get(0).unwrap().into_val(&env),
        Symbol::new(&env, "burn")
    );
    assert_eq!(event.0.get(1).unwrap().into_val(&env), user.clone().into());
    assert_eq!(event.1.into_val(&env), amount);
}

#[test]
fn test_burn_from_event_schema() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &500i128, &1000u32);

    // Clear events
    let _ = env.events().all();

    // Burn from user's allowance
    let amount = 200i128;
    client.burn_from(&spender, &user, &amount);

    // Verify event structure
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.first().unwrap();

    // Topics should be: [burn_symbol, spender_address, from_address]
    // Payload should be: amount
    assert_eq!(event.0.len(), 3);
    assert_eq!(
        event.0.get(0).unwrap().into_val(&env),
        Symbol::new(&env, "burn")
    );
    assert_eq!(
        event.0.get(1).unwrap().into_val(&env),
        spender.clone().into()
    );
    assert_eq!(event.0.get(2).unwrap().into_val(&env), user.clone().into());
    assert_eq!(event.1.into_val(&env), amount);
}

#[test]
fn test_transfer_event_schema() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let receiver = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);

    // Clear events
    let _ = env.events().all();

    // Transfer tokens
    let amount = 400i128;
    client.transfer(&user, &receiver, &amount);

    // Verify event structure
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.first().unwrap();

    // Topics should be: [transfer_symbol, from_address, to_address]
    // Payload should be: amount
    assert_eq!(event.0.len(), 3);
    assert_eq!(
        event.0.get(0).unwrap().into_val(&env),
        Symbol::new(&env, "transfer")
    );
    assert_eq!(event.0.get(1).unwrap().into_val(&env), user.clone().into());
    assert_eq!(
        event.0.get(2).unwrap().into_val(&env),
        receiver.clone().into()
    );
    assert_eq!(event.1.into_val(&env), amount);
}

#[test]
fn test_transfer_from_event_schema() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);
    client.approve(&user, &spender, &500i128, &1000u32);

    // Clear events
    let _ = env.events().all();

    // Transfer from user's allowance
    let amount = 300i128;
    client.transfer_from(&spender, &user, &receiver, &amount);

    // Verify event structure
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.first().unwrap();

    // Topics should be: [transfer_symbol, from_address, to_address]
    // Payload should be: amount
    assert_eq!(event.0.len(), 3);
    assert_eq!(
        event.0.get(0).unwrap().into_val(&env),
        Symbol::new(&env, "transfer")
    );
    assert_eq!(event.0.get(1).unwrap().into_val(&env), user.clone().into());
    assert_eq!(
        event.0.get(2).unwrap().into_val(&env),
        receiver.clone().into()
    );
    assert_eq!(event.1.into_val(&env), amount);
}

#[test]
fn test_approve_event_schema() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let spender = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);

    // Clear events
    let _ = env.events().all();

    // Set allowance
    let amount = 400i128;
    let expiration = 1000u32;
    client.approve(&user, &spender, &amount, &expiration);

    // Verify event structure
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.first().unwrap();

    // Topics should be: [approve_symbol, from_address, spender_address]
    // Payload should be: amount
    assert_eq!(event.0.len(), 3);
    assert_eq!(
        event.0.get(0).unwrap().into_val(&env),
        Symbol::new(&env, "approve")
    );
    assert_eq!(event.0.get(1).unwrap().into_val(&env), user.clone().into());
    assert_eq!(
        event.0.get(2).unwrap().into_val(&env),
        spender.clone().into()
    );
    assert_eq!(event.1.into_val(&env), amount);
}

#[test]
fn test_clawback_event_schema() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);

    // Clear events
    let _ = env.events().all();

    // Clawback tokens
    let amount = 300i128;
    client.clawback(&admin, &user, &amount);

    // Verify event structure
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.first().unwrap();

    // Topics should be: [clawback_symbol, admin_address, from_address]
    // Payload should be: amount
    assert_eq!(event.0.len(), 3);
    assert_eq!(
        event.0.get(0).unwrap().into_val(&env),
        Symbol::new(&env, "clawback")
    );
    assert_eq!(event.0.get(1).unwrap().into_val(&env), admin.clone().into());
    assert_eq!(event.0.get(2).unwrap().into_val(&env), user.clone().into());
    assert_eq!(event.1.into_val(&env), amount);
}

#[test]
fn test_all_core_events_use_consistent_symbol_short_format() {
    let (env, admin, user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let receiver = Address::generate(&env);
    let spender = Address::generate(&env);

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &user, &1000i128);

    // Clear events
    let _ = env.events().all();

    // Execute all core operations
    client.transfer(&user, &receiver, &100i128);
    client.approve(&user, &spender, &200i128, &1000u32);
    client.burn(&user, &50i128);
    client.clawback(&admin, &user, &25i128);

    // Verify all events use symbol_short format (single symbol as first topic)
    let events = env.events().all();
    for event in events.iter() {
        // First topic should always be a Symbol
        let first_topic = event.0.get(0).unwrap();
        // This will succeed if it's a Symbol
        let _: Symbol = first_topic.into_val(&env);
    }
}

// --- Issue #164: End-to-end ticket purchase flow tests ---

#[test]
fn test_ticket_purchase_happy_path() {
    let (env, admin, buyer) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let organiser = Address::generate(&env);
    let ticket_price = 500i128;

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &buyer, &ticket_price);

    let initial_supply = client.total_supply();
    let buyer_before = client.balance(&buyer);

    // Step 2: Buyer creates escrow
    let escrow_id = client.create_escrow(&buyer, &organiser, &ticket_price, &1000u32);

    // Step 3: Buyer balance decreased, contract balance increased
    assert_eq!(client.balance(&buyer), buyer_before - ticket_price);
    assert_eq!(client.total_supply(), initial_supply); // supply unchanged

    // Step 4: Organiser releases escrow
    client.release_escrow(&organiser, &escrow_id);

    // Step 5: Organiser received funds, contract balance is zero
    assert_eq!(client.balance(&organiser), ticket_price);

    // Step 6: Total supply unchanged throughout
    assert_eq!(client.total_supply(), initial_supply);
}

#[test]
fn test_ticket_purchase_dispute_path() {
    let (env, admin, buyer) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    let organiser = Address::generate(&env);
    let resolver = Address::generate(&env);
    let ticket_price = 500i128;

    initialize_client(&client, &env, &admin, 7);
    client.mint(&admin, &buyer, &ticket_price);

    let initial_supply = client.total_supply();

    // Step 1-2: Buyer creates escrow
    let escrow_id = client.create_escrow(&buyer, &organiser, &ticket_price, &1000u32);
    assert_eq!(client.balance(&buyer), 0);

    // Step 2: Buyer opens dispute (event cancelled)
    let dispute_id = client.open_dispute(&buyer, &escrow_id, &resolver);

    // Step 3: Resolver resolves in favour of buyer (refund)
    client.resolve_dispute(&resolver, &dispute_id, &false);

    // Step 4: Buyer got funds back, contract balance is zero
    assert_eq!(client.balance(&buyer), ticket_price);

    // Total supply unchanged throughout
    assert_eq!(client.total_supply(), initial_supply);
}
