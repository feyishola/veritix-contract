#![cfg(test)]

use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env};
use crate::contract::{VeriTixPay, VeriTixPayClient};

struct TestEnv<'a> {
    e: Env,
    client: VeriTixPayClient<'a>,
    from: Address,
    spender: Address,
    to: Address,
}

fn setup() -> TestEnv<'static> {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let from = Address::generate(&e);
    let spender = Address::generate(&e);
    let to = Address::generate(&e);

    TestEnv { e, client, from, spender, to }
}

#[test]
fn test_allowance_valid_at_expiry_ledger() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 10;
    t.client.approve(&t.from, &t.spender, &1000, &expiry);

    t.e.ledger().with_mut(|li| {
        li.sequence = expiry;
    });

    t.client.transfer_from(&t.spender, &t.from, &t.to, &500);
}

#[test]
#[should_panic(expected = "allowance expired")]
fn test_allowance_expired_one_ledger_past() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 10;
    t.client.approve(&t.from, &t.spender, &1000, &expiry);

    t.e.ledger().with_mut(|li| {
        li.sequence = expiry + 1;
    });

    t.client.transfer_from(&t.spender, &t.from, &t.to, &500);
}

#[test]
fn test_allowance_still_valid_one_before_expiry() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 10;
    t.client.approve(&t.from, &t.spender, &1000, &expiry);

    t.e.ledger().with_mut(|li| {
        li.sequence = expiry - 1;
    });

    t.client.transfer_from(&t.spender, &t.from, &t.to, &500);
}

#[test]
fn test_allowance_with_max_u32_expiry() {
    let t = setup();
    let expiry = u32::MAX;
    t.client.approve(&t.from, &t.spender, &1000, &expiry);

    t.e.ledger().with_mut(|li| {
        li.sequence = expiry;
    });

    t.client.transfer_from(&t.spender, &t.from, &t.to, &500);
}