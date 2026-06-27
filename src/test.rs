// ── Existing Test Suite Elements ─────────────────────────────────────────────

#[test]
#[should_panic(expected = "AlreadyFrozen: account is already frozen")]
fn test_freeze_account_panics_if_already_frozen() {
    let env = Env::default();
    let account = Address::generate(&env);

    // First freeze should succeed smoothly
    freeze_account(&env, account.clone());

    // Second freeze must panic and abort execution
    freeze_account(&env, account);
}

#[test]
#[should_panic(expected = "NotFrozen: account is not frozen")]
fn test_unfreeze_account_panics_if_not_frozen() {
    let env = Env::default();
    let account = Address::generate(&env);

    // Account is active by default; unfreezing here must panic instantly
    unfreeze_account(&env, account);
}

// Placeholder declarations matching test dependency references above
fn freeze_account(_e: &Env, _a: Address) {}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    #[should_panic(expected = "Amount must be strictly positive")]
    fn test_transfer_from_rejects_zero_amount() {
        let env = Env::default();
        let spender = env.accounts().generate();
        let from = env.accounts().generate();
        let to = env.accounts().generate();

        // Should panic directly via validation rules
        TokenContract::transfer_from(env, spender, from, to, 0);
    }

    #[test]
    #[should_panic(expected = "Amount must be strictly positive")]
    fn test_transfer_from_rejects_negative_amount() {
        let env = Env::default();
        let spender = env.accounts().generate();
        let from = env.accounts().generate();
        let to = env.accounts().generate();

        TokenContract::transfer_from(env, spender, from, to, -500);
    }

    #[test]
    #[should_panic(expected = "Amount must be strictly positive")]
    fn test_setup_recurring_rejects_zero_amount() {
        let env = Env::default();
        let payer = env.accounts().generate();

        TokenContract::setup_recurring(env, payer, 100, 0);
    }

    mod transfer_frozen {
        use super::*;

        #[test]
        #[should_panic]
        fn test_transfer_from_frozen_sender_panics() {
            let env = Env::default();
            let sender = env.accounts().generate();
            let receiver = env.accounts().generate();

            TokenContract::freeze_account(env.clone(), sender.clone());
            TokenContract::transfer(env, sender, receiver, 100);
        }

        #[test]
        fn test_transfer_to_frozen_receiver_succeeds() {
            let env = Env::default();
            let sender = env.accounts().generate();
            let receiver = env.accounts().generate();

            TokenContract::freeze_account(env.clone(), receiver.clone());
            TokenContract::transfer(env, sender, receiver, 100);
        }

        #[test]
        #[should_panic]
        fn test_transfer_from_frozen_sender_via_transfer_from_panics() {
            let env = Env::default();
            let spender = env.accounts().generate();
            let from = env.accounts().generate();
            let to = env.accounts().generate();

            TokenContract::freeze_account(env.clone(), from.clone());
            TokenContract::transfer_from(env, spender, from, to, 100);
        }

        #[test]
        fn test_clawback_from_frozen_account_succeeds() {
            let env = Env::default();
            let from = env.accounts().generate();
            let to = env.accounts().generate();

            TokenContract::freeze_account(env.clone(), from.clone());
            TokenContract::clawback(env, from, to, 100);
        }
    }
}