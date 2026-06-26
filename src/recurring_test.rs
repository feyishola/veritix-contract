#![cfg(test)]

use soroban_sdk::{testutils::{Address as _, Ledger, LedgerInfo}, Address, Env};
use crate::contract::{VeriTixPay, VeriTixPayClient};

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

#[test]
fn test_execute_on_exact_boundary_succeeds() {
    let t = setup();
    let interval = 100;
    let id = t.client.create_recurring(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &10,
        &interval,
        &5,
    );

    t.e.ledger().with_mut(|li: &mut LedgerInfo| {
        li.sequence_number += interval;
    });

    t.client.execute_recurring(&id);
    let tc = soroban_sdk::token::Client::new(&t.e, &t.token);
    assert_eq!(tc.balance(&t.beneficiary), 10);
}

#[test]
#[should_panic(expected = "not time to charge yet")]
fn test_execute_one_ledger_before_boundary_panics() {
    let t = setup();
    let interval = 100;
    let id = t.client.create_recurring(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &10,
        &interval,
        &5,
    );

    t.e.ledger().with_mut(|li: &mut LedgerInfo| {
        li.sequence_number += interval - 1;
    });

    t.client.execute_recurring(&id);
}

#[test]
fn test_execute_one_ledger_after_boundary_succeeds() {
    let t = setup();
    let interval = 100;
    let id = t.client.create_recurring(
        &t.depositor,
        &t.beneficiary,
        &t.token,
        &10,
        &interval,
        &5,
    );

    t.e.ledger().with_mut(|li: &mut LedgerInfo| {
        li.sequence_number += interval + 1;
    });

    t.client.execute_recurring(&id);
    let tc = soroban_sdk::token::Client::new(&t.e, &t.token);
    assert_eq!(tc.balance(&t.beneficiary), 10);
}