use crate::test::{create_client, initialize_client, setup};
use soroban_sdk::{Address, Env, Vec};

#[test]
#[should_panic]
fn test_mint_batch_with_one_frozen_recipient_panics_and_reverts_all() {
    let (env, admin, _) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    initialize_client(&client, &env, &admin, 7);

    let addresses: Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let frozen_address = addresses.get(1).unwrap().unwrap();
    let mint_amount = 100i128;

    client.freeze(&frozen_address);

    // This should panic and revert
    client.mint_batch(&admin, &addresses, &mint_amount);

    // Verify no balances were changed
    for addr in addresses.iter() {
        assert_eq!(client.balance(&addr.unwrap()), 0);
    }
}

#[test]
#[should_panic]
fn test_clawback_batch_with_one_insufficient_balance_panics() {
    let (env, admin, _) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    initialize_client(&client, &env, &admin, 7);

    let addresses: Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let mint_amount = 100i128;

    // Mint to all addresses except one
    for i in 0..addresses.len() {
        if i != 1 {
            client.mint(&admin, &addresses.get(i).unwrap().unwrap(), &mint_amount);
        }
    }

    // This should panic
    client.clawback_batch(&admin, &addresses, &mint_amount);
}

#[test]
#[should_panic]
fn test_freeze_batch_with_already_frozen_address_panics() {
    let (env, admin, _) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    initialize_client(&client, &env, &admin, 7);

    let addresses: Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let frozen_address = addresses.get(1).unwrap().unwrap();

    client.freeze(&frozen_address);

    // This should panic
    client.freeze_batch(&admin, &addresses);
}