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