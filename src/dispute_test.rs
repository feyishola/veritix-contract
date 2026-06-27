#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::contract::{VeriTixPay, VeriTixPayClient};

struct TestEnv<'a> {
    e: Env,
    client: VeriTixPayClient<'a>,
    depositor: Address,
    beneficiary: Address,
    token: Address,
    admin: Address,
}

fn setup() -> TestEnv<'static> {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let admin = Address::generate(&e);
    let token = e.register_stellar_asset_contract(depositor.clone());

    soroban_sdk::token::StellarAssetClient::new(&e, &token).mint(&depositor, &50_000);

    client.initialize(&admin);

    TestEnv { e, client, depositor, beneficiary, token, admin }
}

#[test]
#[should_panic(expected = "already settled")]
fn test_release_escrow_after_dispute_resolved_for_beneficiary_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;
    let escrow_id = t.client.create_escrow(&t.depositor, &t.beneficiary, &t.token, &1000, &expiry, &soroban_sdk::Bytes::new(&t.e));

    t.client.raise_dispute(&t.depositor, &escrow_id);
    t.client.resolve_dispute(&t.admin, &escrow_id, &t.beneficiary);
    t.client.release_escrow(&t.depositor, &escrow_id);
}

#[test]
#[should_panic(expected = "already settled")]
fn test_refund_escrow_after_dispute_resolved_for_depositor_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;
    let escrow_id = t.client.create_escrow(&t.depositor, &t.beneficiary, &t.token, &1000, &expiry, &soroban_sdk::Bytes::new(&t.e));

    t.client.raise_dispute(&t.depositor, &escrow_id);
    t.client.resolve_dispute(&t.admin, &escrow_id, &t.depositor);
    t.client.refund_escrow(&t.depositor, &escrow_id);
}

#[test]
#[should_panic(expected = "InvalidState")]
fn test_open_dispute_after_resolved_panics() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;
    let escrow_id = t.client.create_escrow(&t.depositor, &t.beneficiary, &t.token, &1000, &expiry, &soroban_sdk::Bytes::new(&t.e));

    t.client.raise_dispute(&t.depositor, &escrow_id);
    t.client.resolve_dispute(&t.admin, &escrow_id, &t.depositor);
    t.client.raise_dispute(&t.depositor, &escrow_id);
}