#[cfg(test)]
mod recurring_tests {
    use soroban_sdk::{
        testutils::{Address as _, Events as _, Ledger as _},
        Address, Env,
    };

    use crate::balance::read_balance;
    use crate::contract::{VeritixToken, VeritixTokenClient};
    use crate::recurring::{cancel_recurring, execute_recurring, get_next_execution_ledger, get_recurring, is_executable, pause_recurring, setup_recurring};
    use crate::storage_types::{read_counter, DataKey};

    fn setup_env() -> Env {
        let e = Env::default();
        e.mock_all_auths();
        e
    }

    fn fund_and_setup(
        e: &Env,
        contract_id: &Address,
        amount: i128,
        interval: u32,
    ) -> (Address, Address, u32) {
        let payer = Address::generate(e);
        let payee = Address::generate(e);
        let mut id = 0u32;
        e.as_contract(contract_id, || {
            crate::balance::receive_balance(e, payer.clone(), amount);
            id = setup_recurring(e, payer.clone(), payee.clone(), amount, interval);
        });
        (payer, payee, id)
    }

    // Ensures that creating a recurring payment where payer == payee is
    // rejected — prevents degenerate self-payment schedules.
    #[test]
    #[should_panic(expected = "InvalidRecurring: payer and payee cannot be the same address")]
    fn test_setup_recurring_same_address_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let addr = Address::generate(&e);

        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, addr.clone(), 500);
            setup_recurring(&e, addr.clone(), addr.clone(), 500, 100);
        });
    }

    // Verifies that setup_recurring stores a record with correct payer, payee,
    // amount, interval, and active state. If this fails, recurring payment
    // setup is broken.
    #[test]
    fn test_setup_stores_record() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            let r = get_recurring(&e, id);
            assert_eq!(r.payer, payer);
            assert_eq!(r.payee, payee);
            assert_eq!(r.amount, 500);
            assert_eq!(r.interval, 100);
            assert!(r.active);
        });
    }

    // Happy-path execute: advances ledger past the interval and verifies that
    // funds move from payer to payee correctly.
    #[test]
    fn test_execute_transfers_funds() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            // Advance ledger past the interval.
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 101);
            execute_recurring(&e, id);
            assert_eq!(read_balance(&e, payee.clone()), 500);
            assert_eq!(read_balance(&e, payer.clone()), 0);
        });
    }

    // Ensures that executing a recurring payment before the interval has
    // elapsed panics — prevents early withdrawal of funds.
    #[test]
    #[should_panic(expected = "interval has not elapsed")]
    fn test_execute_too_early_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            // Only advance by 50 — not enough.
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 50);
            execute_recurring(&e, id);
        });
    }

    // Verifies that cancelling a recurring payment deactivates the record
    // and prevents future executions.
    #[test]
    fn test_cancel_deactivates_record() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            cancel_recurring(&e, payer.clone(), id);
            let r = get_recurring(&e, id);
            assert!(!r.active);
        });
    }

    // Ensures that only the payer can cancel a recurring payment — a third
    // party hacker must be rejected with "unauthorized".
    #[test]
    #[should_panic(expected = "unauthorized")]
    fn test_cancel_unauthorized_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);
        let hacker = Address::generate(&e);

        e.as_contract(&contract_id, || {
            cancel_recurring(&e, hacker, id);
        });
    }

    // Ensures that executing a cancelled recurring payment panics — prevents
    // funds from being transferred after the payer has deactivated the plan.
    #[test]
    #[should_panic(expected = "recurring payment is not active")]
    fn test_execute_after_cancel_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        e.as_contract(&contract_id, || {
            cancel_recurring(&e, payer.clone(), id);
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 200);
            execute_recurring(&e, id);
        });
    }

    // Ensures that creating a recurring payment with zero amount is rejected.
    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_recurring_zero_amount_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let payer = Address::generate(&e);
        let payee = Address::generate(&e);
        e.as_contract(&contract_id, || {
            setup_recurring(&e, payer.clone(), payee.clone(), 0, 100);
        });
    }

    // Ensures that creating a recurring payment with zero interval is rejected.
    #[test]
    #[should_panic(expected = "interval must be positive")]
    fn test_recurring_zero_interval_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let payer = Address::generate(&e);
        let payee = Address::generate(&e);
        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, payer.clone(), 500);
            setup_recurring(&e, payer.clone(), payee.clone(), 500, 0);
        });
    }

    // Verifies that setup_recurring requires the payee's authorization in
    // addition to the payer's — both parties must consent to the schedule.
    #[test]
    fn test_recurring_payee_auth_required() {
        let e = Env::default();
        let contract_id = e.register_contract(None, VeritixToken);
        let client = VeritixTokenClient::new(&e, &contract_id);
        let payer = Address::generate(&e);
        let payee = Address::generate(&e);

        e.mock_all_auths();
        client.setup_recurring(&payer, &payee, &500i128, &100u32);

        // Payee must be among the authorized signers
        let auths = e.auths();
        let payee_authorized = auths.iter().any(|(addr, _)| addr == &payee);
        assert!(payee_authorized, "payee must authorize setup_recurring");
    }

    // Ensures that execute_recurring panics when the payer has insufficient
    // balance to cover the recurring amount.
    #[test]
    #[should_panic(expected = "InsufficientBalance")]
    fn test_execute_recurring_insufficient_balance_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        // Fund payer with less than the recurring amount.
        let (payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);
        e.as_contract(&contract_id, || {
            // Drain the payer balance so they can no longer cover the charge.
            crate::balance::spend_balance(&e, payer.clone(), 500);
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 101);
            execute_recurring(&e, id);
        });
    }

    // Verifies that a recurring payment can be executed multiple times as long
    // as sufficient balance remains and intervals elapse between executions.
    #[test]
    fn test_multiple_executions() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        // Fund payer with enough for two charges.
        let (payer, payee, id) = fund_and_setup(&e, &contract_id, 1_000, 100);
        // Give payer extra balance for second charge.
        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, payer.clone(), 1_000);
        });

        e.as_contract(&contract_id, || {
            let start = e.ledger().sequence();

            e.ledger().with_mut(|l| l.sequence_number = start + 101);
            execute_recurring(&e, id);
            assert_eq!(read_balance(&e, payee.clone()), 1_000);

            e.ledger().with_mut(|l| l.sequence_number = start + 202);
            execute_recurring(&e, id);
            assert_eq!(read_balance(&e, payee.clone()), 2_000);
        });
    }

    // Verifies that both payer and payee appear in the auths for
    // setup_recurring, confirming the dual-authorization requirement.
    #[test]
    fn test_setup_recurring_requires_both_payer_and_payee_auth() {
        let e = Env::default();
        let contract_id = e.register_contract(None, VeritixToken);
        let client = VeritixTokenClient::new(&e, &contract_id);
        let payer = Address::generate(&e);
        let payee = Address::generate(&e);
        let amount = 500i128;
        let interval = 100u32;

        e.mock_all_auths();
        client.setup_recurring(&payer, &payee, &amount, &interval);

        // Both payer and payee must have authorized the setup_recurring call.
        let auths = e.auths();
        assert_eq!(auths.len(), 2, "expected both payer and payee auth");
        assert_eq!(auths[0].0, payer, "first auth should be payer");
        assert_eq!(auths[1].0, payee, "second auth should be payee");
    }

    // Ensures that executing a recurring payment preserves the supply invariant
    // (sum of balances == total supply) before and after execution.
    #[test]
    fn test_recurring_execute_preserves_supply() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, payee, id) = fund_and_setup(&e, &contract_id, 1_000, 100);

        e.as_contract(&contract_id, || {
            crate::balance::increase_supply(&e, 1_000);

            let assert_invariant = || {
                let sum = read_balance(&e, payer.clone()) + read_balance(&e, payee.clone());
                assert_eq!(crate::balance::read_total_supply(&e), sum);
            };

            assert_invariant();

            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 101);
            execute_recurring(&e, id);
            assert_invariant();

            // Fund payer for a second execution
            crate::balance::receive_balance(&e, payer.clone(), 1_000);
            crate::balance::increase_supply(&e, 1_000);
            assert_invariant();

            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 101);
            execute_recurring(&e, id);
            assert_invariant();
        });
    }

    // --- Issue #162: Event emission tests ---

    // Verifies that setup_recurring emits a single event with
    // (recurring_setup, payer) topics and (payee, amount) as data.
    #[test]
    fn test_setup_recurring_emits_event() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_payer, _payee, _id) = fund_and_setup(&e, &contract_id, 500, 100);

        let events = e.events().all();
        assert_eq!(events.len(), 1);
        // Topics: (recurring_setup, payer), data: (payee, amount)
        assert_eq!(events.first().unwrap().1.len(), 2);
    }

    // Verifies that execute_recurring emits a single event with
    // (recurring_executed, recurring_id) topics and amount as data.
    #[test]
    fn test_execute_recurring_emits_event() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        // Clear setup event
        let _ = e.events().all();

        e.as_contract(&contract_id, || {
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 101);
            execute_recurring(&e, id);
        });

        let events = e.events().all();
        assert_eq!(events.len(), 1);
        // Topics: (recurring_executed, recurring_id), data: amount
        assert_eq!(events.first().unwrap().1.len(), 2);
    }

    // Verifies that cancel_recurring emits a single event with
    // (recurring_cancelled, recurring_id, caller) topics.
    #[test]
    fn test_cancel_recurring_emits_event() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, _payee, id) = fund_and_setup(&e, &contract_id, 500, 100);

        let _ = e.events().all();

        e.as_contract(&contract_id, || {
            cancel_recurring(&e, payer.clone(), id);
        });

        let events = e.events().all();
        assert_eq!(events.len(), 1);
        // Topics: (recurring_cancelled, recurring_id, caller), data: ()
        assert_eq!(events.first().unwrap().1.len(), 3);
    }

    // --- Recurring counter tests ---

    // Ensures the recurring counter starts at zero before any payments are set up.
    #[test]
    fn test_recurring_count_starts_at_zero() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);

        e.as_contract(&contract_id, || {
            let count = read_counter(&e, &DataKey::RecurringCount);
            assert_eq!(count, 0);
        });
    }

    // Verifies the recurring counter increments correctly with IDs 1, 2, 3
    // across multiple setup calls — no ID gaps or collisions.
    #[test]
    fn test_recurring_count_increments_on_setup() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let payer1 = Address::generate(&e);
        let payee1 = Address::generate(&e);
        let payer2 = Address::generate(&e);
        let payee2 = Address::generate(&e);
        let payer3 = Address::generate(&e);
        let payee3 = Address::generate(&e);

        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, payer1.clone(), 1000);
            crate::balance::receive_balance(&e, payer2.clone(), 1000);
            crate::balance::receive_balance(&e, payer3.clone(), 1000);

            // Before any recurring payments
            assert_eq!(read_counter(&e, &DataKey::RecurringCount), 0);

            // Setup first recurring
            let id = setup_recurring(&e, payer1.clone(), payee1.clone(), 500, 100);
            assert_eq!(id, 1);
            assert_eq!(read_counter(&e, &DataKey::RecurringCount), 1);

            // Setup second recurring
            let id = setup_recurring(&e, payer2.clone(), payee2.clone(), 500, 100);
            assert_eq!(id, 2);
            assert_eq!(read_counter(&e, &DataKey::RecurringCount), 2);

            // Setup third recurring
            let id = setup_recurring(&e, payer3.clone(), payee3.clone(), 500, 100);
            assert_eq!(id, 3);
            assert_eq!(read_counter(&e, &DataKey::RecurringCount), 3);
        });
    }

    // Ensures that creating a recurring payment with zero interval is rejected
    // with "InvalidInterval" — matches the updated panic string convention.
    #[test]
    #[should_panic(expected = "InvalidInterval")]
    fn test_setup_recurring_zero_interval_panics() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let payer = Address::generate(&e);
        let payee = Address::generate(&e);

        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, payer.clone(), 500);
            setup_recurring(&e, payer.clone(), payee.clone(), 500, 0);
        });
    }

    #[test]
    fn test_get_next_execution_ledger_active() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_, _, id) = fund_and_setup(&e, &contract_id, 500, 100);
        e.as_contract(&contract_id, || {
            let r = get_recurring(&e, id);
            let next = get_next_execution_ledger(&e, id);
            assert_eq!(next, r.last_charged_ledger + 100);
        });
    }

    #[test]
    fn test_get_next_execution_ledger_paused_returns_max() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, _, id) = fund_and_setup(&e, &contract_id, 500, 100);
        e.as_contract(&contract_id, || {
            pause_recurring(&e, payer.clone(), id);
            assert_eq!(get_next_execution_ledger(&e, id), u32::MAX);
        });
    }

    #[test]
    fn test_get_next_execution_ledger_missing_returns_max() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        e.as_contract(&contract_id, || {
            assert_eq!(get_next_execution_ledger(&e, 999), u32::MAX);
        });
    }

    #[test]
    fn test_is_executable_active_and_due() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_, _, id) = fund_and_setup(&e, &contract_id, 500, 100);
        e.as_contract(&contract_id, || {
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 101);
            assert!(is_executable(&e, id));
        });
    }

    #[test]
    fn test_is_executable_active_not_due() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (_, _, id) = fund_and_setup(&e, &contract_id, 500, 100);
        e.as_contract(&contract_id, || {
            assert!(!is_executable(&e, id));
        });
    }

    #[test]
    fn test_is_executable_paused() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let (payer, _, id) = fund_and_setup(&e, &contract_id, 500, 100);
        e.as_contract(&contract_id, || {
            pause_recurring(&e, payer.clone(), id);
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 200);
            assert!(!is_executable(&e, id));
        });
    }

    #[test]
    fn test_is_executable_missing() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        e.as_contract(&contract_id, || {
            assert!(!is_executable(&e, 999));
        });
    }

    #[test]
    fn test_recurring_payee_is_contract_address() {
        let e = setup_env();
        let contract_id = e.register_contract(None, VeritixToken);
        let payer = Address::generate(&e);

        // Register a second VeritixToken contract as the payee
        let payee_contract_id = e.register_contract(None, VeritixToken);
        let payee_client = crate::contract::VeritixTokenClient::new(&e, &payee_contract_id);
        let payee_admin = Address::generate(&e);
        payee_client.initialize(
            &payee_admin,
            &soroban_sdk::String::from_str(&e, "PayeeToken"),
            &soroban_sdk::String::from_str(&e, "PAY"),
            &7u32,
        );

        let mut recurring_id = 0u32;
        e.as_contract(&contract_id, || {
            crate::balance::receive_balance(&e, payer.clone(), 1_000);
            crate::balance::increase_supply(&e, 1_000);

            // Use the payee contract address as the payee
            recurring_id = setup_recurring(&e, payer.clone(), payee_contract_id.clone(), 500, 100);
        });

        e.as_contract(&contract_id, || {
            e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 101);
            execute_recurring(&e, recurring_id);
        });

        // Verify the payee contract received the tokens
        assert_eq!(payee_client.balance(&payee_contract_id), 500i128);
    }
}
