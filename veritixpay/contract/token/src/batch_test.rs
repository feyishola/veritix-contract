#[cfg(test)]
mod batch_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
    use crate::balance::read_balance;
    use crate::batch::{mint_batch, transfer_batch, BatchEntry};
    use crate::contract::VeritixToken;

    fn setup_env() -> Env {
        let e = Env::default();
        e.mock_all_auths();
        e
    }

    #[test]
    fn test_mint_batch_credits_all_recipients() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let r1 = Address::generate(&e);
        let r2 = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            recs.push_back(BatchEntry { address: r1.clone(), amount: 500 });
            recs.push_back(BatchEntry { address: r2.clone(), amount: 300 });
            mint_batch(&e, admin.clone(), recs);
            assert_eq!(read_balance(&e, r1.clone()), 500);
            assert_eq!(read_balance(&e, r2.clone()), 300);
        });
    }

    #[test]
    fn test_transfer_batch_distributes_correctly() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let from = Address::generate(&e);
        let r1 = Address::generate(&e);
        let r2 = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::balance::receive_balance(&e, from.clone(), 1000);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            recs.push_back(BatchEntry { address: r1.clone(), amount: 400 });
            recs.push_back(BatchEntry { address: r2.clone(), amount: 600 });
            transfer_batch(&e, from.clone(), recs);
            assert_eq!(read_balance(&e, r1.clone()), 400);
            assert_eq!(read_balance(&e, r2.clone()), 600);
            assert_eq!(read_balance(&e, from.clone()), 0);
        });
    }

    #[test]
    #[should_panic(expected = "BatchLimit")]
    fn test_mint_batch_rejects_over_50() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            for _ in 0..51 {
                recs.push_back(BatchEntry { address: Address::generate(&e), amount: 10 });
            }
            mint_batch(&e, admin.clone(), recs);
        });
    }

    #[test]
    #[should_panic(expected = "BatchLimit")]
    fn test_transfer_batch_rejects_over_50() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let from = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::balance::receive_balance(&e, from.clone(), 100_000);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            for _ in 0..51 {
                recs.push_back(BatchEntry { address: Address::generate(&e), amount: 1 });
            }
            transfer_batch(&e, from.clone(), recs);
        });
    }
}
