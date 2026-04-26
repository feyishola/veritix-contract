#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
use crate::contract::{VeriTixPay, VeriTixPayClient};

fn setup() -> (Env, VeriTixPayClient<'static>, Address, Address, Address) {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let token = e.register_stellar_asset_contract(depositor.clone());

    // Fund the depositor
    let token_client = soroban_sdk::token::StellarAssetClient::new(&e, &token);
    token_client.mint(&depositor, &10_000);

    (e, client, depositor, beneficiary, token)
}

#[test]
fn test_create_escrow_indexes_both_parties() {
    let (e, client, depositor, beneficiary, token) = setup();
    let expiry = e.ledger().sequence() + 1000;

    let id = client.create_escrow(&depositor, &beneficiary, &token, &500, &expiry);

    // Depositor index contains this escrow
    let by_depositor = client.get_escrows_by_depositor(&depositor);
    assert_eq!(by_depositor.len(), 1);
    assert_eq!(by_depositor.get(0).unwrap(), id);

    // Beneficiary index contains this escrow
    let by_beneficiary = client.get_escrows_by_beneficiary(&beneficiary);
    assert_eq!(by_beneficiary.len(), 1);
    assert_eq!(by_beneficiary.get(0).unwrap(), id);
}

#[test]
fn test_beneficiary_index_accumulates_multiple_escrows() {
    let (e, client, depositor, beneficiary, token) = setup();
    let expiry = e.ledger().sequence() + 1000;

    client.create_escrow(&depositor, &beneficiary, &token, &100, &expiry);
    client.create_escrow(&depositor, &beneficiary, &token, &200, &expiry);
    client.create_escrow(&depositor, &beneficiary, &token, &300, &expiry);

    let list = client.get_escrows_by_beneficiary(&beneficiary);
    assert_eq!(list.len(), 3);
}

#[test]
fn test_unrelated_address_gets_empty_list() {
    let (e, client, ..) = setup();
    let stranger = Address::generate(&e);
    let list = client.get_escrows_by_beneficiary(&stranger);
    assert_eq!(list.len(), 0);
}

#[test]
fn test_depositor_does_not_appear_in_beneficiary_index() {
    let (e, client, depositor, beneficiary, token) = setup();
    let expiry = e.ledger().sequence() + 1000;

    client.create_escrow(&depositor, &beneficiary, &token, &500, &expiry);

    // Depositor's beneficiary list should be empty
    let depositor_as_beneficiary = client.get_escrows_by_beneficiary(&depositor);
    assert_eq!(depositor_as_beneficiary.len(), 0);
}
