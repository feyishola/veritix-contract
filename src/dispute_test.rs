#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::contract::{VeriTixPay, VeriTixPayClient};

struct TestEnv<'a> {
    e: Env,
    client: VeriTixPayClient<'a>,
    depositor: Address,
    beneficiary: Address,
    token: Address,
    arbiter: Address,
}

fn setup() -> TestEnv<'static> {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let depositor = Address::generate(&e);
    let beneficiary = Address::generate(&e);
    let arbiter = Address::generate(&e);
    let token = e.register_stellar_asset_contract(depositor.clone());

    soroban_sdk::token::StellarAssetClient::new(&e, &token).mint(&depositor, &50_000);
    client.set_arbiter(&arbiter);

    TestEnv { e, client, depositor, beneficiary, token, arbiter }
}

#[test]
fn test_dispute_on_one_escrow_does_not_affect_another() {
    let t = setup();
    let expiry = t.e.ledger().sequence() + 1000;

    let depositor1 = t.depositor;
    let beneficiary1 = t.beneficiary;

    let depositor2 = Address::generate(&t.e);
    let beneficiary2 = Address::generate(&t.e);
    soroban_sdk::token::StellarAssetClient::new(&t.e, &t.token).mint(&depositor2, &1000);

    let escrow1_id = t.client.create_escrow(
        &depositor1,
        &beneficiary1,
        &t.token,
        &100,
        &expiry,
        &crate::escrow_test::empty_memo(&t.e),
    );

    let escrow2_id = t.client.create_escrow(
        &depositor2,
        &beneficiary2,
        &t.token,
        &200,
        &expiry,
        &crate::escrow_test::empty_memo(&t.e),
    );

    t.client.raise_dispute(&depositor1, &escrow1_id);
    t.client.resolve_dispute(&t.arbiter, &escrow1_id, &beneficiary1);

    let escrow2 = t.client.get_escrow(&escrow2_id);
    assert!(!escrow2.released);

    t.client.release_escrow(&beneficiary2, &escrow2_id);

    let tc = soroban_sdk::token::Client::new(&t.e, &t.token);
    assert_eq!(tc.balance(&beneficiary1), 100);
    assert_eq!(tc.balance(&beneficiary2), 200);
}
