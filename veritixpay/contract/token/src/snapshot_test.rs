#[cfg(test)]
mod snapshot_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
    use crate::balance::{read_balance, receive_balance};
    use crate::contract::VeritixToken;

    fn setup_env() -> Env {
        let e = Env::default();
        e.mock_all_auths();
        e
    }

    #[test]
    fn test_take_snapshot_records_balances() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);

        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            receive_balance(&e, user1.clone(), 500);
            receive_balance(&e, user2.clone(), 300);

            let mut addresses: Vec<Address> = Vec::new(&e);
            addresses.push_back(user1.clone());
            addresses.push_back(user2.clone());

            let snapshot_id = crate::snapshot::take_snapshot(&e, admin.clone(), addresses);

            assert_eq!(crate::snapshot::get_snapshot_balance(&e, snapshot_id, user1.clone()), 500);
            assert_eq!(crate::snapshot::get_snapshot_balance(&e, snapshot_id, user2.clone()), 300);
            assert_eq!(crate::snapshot::get_snapshot_ledger(&e, snapshot_id), e.ledger().sequence());
        });
    }

    #[test]
    fn test_take_snapshot_increments_id() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);

        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            let addresses: Vec<Address> = Vec::new(&e);

            let id1 = crate::snapshot::take_snapshot(&e, admin.clone(), addresses.clone());
            let id2 = crate::snapshot::take_snapshot(&e, admin.clone(), addresses);
            assert_eq!(id1, 1);
            assert_eq!(id2, 2);
        });
    }

    #[test]
    fn test_get_snapshot_balance_returns_zero_for_unknown_address() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);

        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            let addresses: Vec<Address> = Vec::new(&e);
            let snapshot_id = crate::snapshot::take_snapshot(&e, admin.clone(), addresses);
            let unknown = Address::generate(&e);
            assert_eq!(crate::snapshot::get_snapshot_balance(&e, snapshot_id, unknown), 0);
        });
    }
}
