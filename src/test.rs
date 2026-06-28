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
}

#[cfg(test)]
mod airdrop_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Address, Vec};
    use crate::contract::{VeriTixPay, VeriTixPayClient};
    use crate::storage_types::DataKey;

    fn setup_airdrop() -> (Env, VeriTixPayClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, VeriTixPay);
        let client = VeriTixPayClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let token = env.register_stellar_asset_contract(admin.clone());
        
        // init admin
        env.as_contract(&contract_id, || {
            env.storage().persistent().set(&DataKey::Admin, &admin);
        });

        soroban_sdk::token::StellarAssetClient::new(&env, &token).mint(&admin, &100_000);
        
        (env, client, admin, token)
    }

    #[test]
    fn test_airdrop_success() {
        let (env, client, admin, token) = setup_airdrop();
        
        let holder1 = Address::generate(&env);
        let holder2 = Address::generate(&env);

        let mut holders: Vec<(Address, i128)> = Vec::new(&env);
        holders.push_back((holder1.clone(), 300));
        holders.push_back((holder2.clone(), 700));

        env.as_contract(&client.address, || {
            env.storage().persistent().set(&DataKey::HolderSet, &holders);
        });

        client.airdrop(&admin, &10_000, &token);

        let tc = soroban_sdk::token::Client::new(&env, &token);
        assert_eq!(tc.balance(&holder1), 3000); // 30% of 10k
        assert_eq!(tc.balance(&holder2), 7000); // 70% of 10k
    }

    #[test]
    #[should_panic(expected = "maximum 50 holders per airdrop call")]
    fn test_airdrop_too_many_holders() {
        let (env, client, admin, token) = setup_airdrop();
        let mut holders: Vec<(Address, i128)> = Vec::new(&env);
        
        for _ in 0..51 {
            holders.push_back((Address::generate(&env), 10));
        }

        env.as_contract(&client.address, || {
            env.storage().persistent().set(&DataKey::HolderSet, &holders);
        });

        client.airdrop(&admin, &10_000, &token);
    }
}