use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

use crate::balance::read_balance;
use crate::contract::VeritixToken;
use crate::splitter::{cancel_split, create_split, distribute, get_split, SplitRecipient};

fn setup_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

fn make_recipients(e: &Env, shares: &[(Address, u32)]) -> Vec<SplitRecipient> {
    let mut v = Vec::new(e);
    for (addr, bps) in shares {
        v.push_back(SplitRecipient {
            address: addr.clone(),
            share_bps: *bps,
        });
    }
    v
}

#[test]
fn test_create_split_stores_record() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 5000), (r2.clone(), 5000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        let record = get_split(&e, split_id);
        assert_eq!(record.sender, sender);
        assert_eq!(record.total_amount, 1000);
        assert!(!record.distributed);
        assert!(!record.cancelled);
    });
}

#[test]
fn test_distribute_two_recipients_equal_split() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 5000), (r2.clone(), 5000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        distribute(&e, sender.clone(), split_id);
        assert_eq!(read_balance(&e, r1.clone()), 500);
        assert_eq!(read_balance(&e, r2.clone()), 500);
        assert!(get_split(&e, split_id).distributed);
    });
}

#[test]
fn test_cancel_split_refunds_sender_and_marks_record() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);

        assert_eq!(read_balance(&e, sender.clone()), 0);

        cancel_split(&e, sender.clone(), split_id);

        let record = get_split(&e, split_id);
        assert!(record.cancelled);
        assert!(!record.distributed);
        assert_eq!(read_balance(&e, sender.clone()), 1000);
        assert_eq!(read_balance(&e, r1.clone()), 0);
    });
}

#[test]
fn test_distribute_rounding_dust_goes_to_last_recipient() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);
    let r3 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 10);
        // 3333 + 3333 + 3334 = 10000 bps; 10 units → 3 + 3 + 4
        let recipients = make_recipients(
            &e,
            &[(r1.clone(), 3333), (r2.clone(), 3333), (r3.clone(), 3334)],
        );
        let split_id = create_split(&e, sender.clone(), recipients, 10);
        distribute(&e, sender.clone(), split_id);
        assert_eq!(read_balance(&e, r1.clone()), 3);
        assert_eq!(read_balance(&e, r2.clone()), 3);
        assert_eq!(read_balance(&e, r3.clone()), 4);
    });
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_distribute_unauthorized_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let hacker = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        distribute(&e, hacker.clone(), split_id);
    });
}

#[test]
#[should_panic(expected = "already distributed")]
fn test_double_distribute_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        distribute(&e, sender.clone(), split_id);
        distribute(&e, sender.clone(), split_id);
    });
}

#[test]
#[should_panic(expected = "already distributed")]
fn test_cancel_after_distribute_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        distribute(&e, sender.clone(), split_id);
        cancel_split(&e, sender.clone(), split_id);
    });
}

#[test]
#[should_panic(expected = "split cancelled")]
fn test_distribute_after_cancel_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        let split_id = create_split(&e, sender.clone(), recipients, 1000);
        cancel_split(&e, sender.clone(), split_id);
        distribute(&e, sender.clone(), split_id);
    });
}

#[test]
#[should_panic(expected = "recipients list cannot be empty")]
fn test_create_split_rejects_empty_recipients() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients: Vec<SplitRecipient> = Vec::new(&e);
        create_split(&e, sender.clone(), recipients, 1000);
    });
}

#[test]
#[should_panic(expected = "recipient share_bps cannot be zero")]
fn test_create_split_rejects_zero_share_recipient() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 10000), (r2.clone(), 0)]);
        create_split(&e, sender.clone(), recipients, 1000);
    });
}

#[test]
#[should_panic(expected = "duplicate recipient address")]
fn test_create_split_rejects_duplicate_recipients() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        let recipients = make_recipients(&e, &[(r1.clone(), 5000), (r1.clone(), 5000)]);
        create_split(&e, sender.clone(), recipients, 1000);
    });
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_create_split_rejects_non_positive_amount() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);

    e.as_contract(&contract_id, || {
        let recipients = make_recipients(&e, &[(r1.clone(), 10000)]);
        create_split(&e, sender.clone(), recipients, 0);
    });
}

#[test]
#[should_panic(expected = "TooManyRecipients: maximum 20 recipients allowed")]
fn test_create_split_too_many_recipients_panics() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), 1000);
        // Build 21 recipients each with equal share — total bps won't matter since
        // the cap check fires first.
        let mut pairs: soroban_sdk::Vec<SplitRecipient> = soroban_sdk::Vec::new(&e);
        for _ in 0..21 {
            pairs.push_back(SplitRecipient {
                address: Address::generate(&e),
                share_bps: 476, // approximate; cap check fires before bps validation
            });
        }
        create_split(&e, sender.clone(), pairs, 1000);
    });
}

#[test]
fn test_split_create_and_distribute_preserves_supply() {
    let e = setup_env();
    let contract_id = e.register_contract(None, VeritixToken);
    let sender = Address::generate(&e);
    let r1 = Address::generate(&e);
    let r2 = Address::generate(&e);
    let amount = 1_000i128;

    e.as_contract(&contract_id, || {
        crate::balance::receive_balance(&e, sender.clone(), amount);
        crate::balance::increase_supply(&e, amount);

        let contract_addr = e.current_contract_address();
        let all = [sender.clone(), r1.clone(), r2.clone(), contract_addr.clone()];

        let assert_invariant = |addrs: &[Address]| {
            let sum = addrs.iter().fold(0i128, |s, a| s + read_balance(&e, a.clone()));
            assert_eq!(crate::balance::read_total_supply(&e), sum);
        };

        assert_invariant(&all);

        let recipients = make_recipients(&e, &[(r1.clone(), 5000), (r2.clone(), 5000)]);
        let split_id = create_split(&e, sender.clone(), recipients, amount);
        assert_invariant(&all);

        distribute(&e, sender.clone(), split_id);
        assert_invariant(&all);
    });
}
