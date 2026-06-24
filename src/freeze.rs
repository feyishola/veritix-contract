// Assuming a standard storage context layout or wrapper struct
pub fn freeze_account(env: &Env, account_id: Address) {
    // 1. Fetch the current status from storage
    let is_frozen: bool = env.storage().instance().get(&account_id).unwrap_or(false);

    // 2. Strict Check: Panic on redundant execution state
    if is_frozen {
        panic!("AlreadyFrozen: account is already frozen");
    }

    // 3. Write state only if valid
    env.storage().instance().set(&account_id, &true);
}

pub fn unfreeze_account(env: &Env, account_id: Address) {
    // 1. Fetch the current status from storage
    let is_frozen: bool = env.storage().instance().get(&account_id).unwrap_or(false);

    // 2. Strict Check: Panic if trying to unfreeze an already active account
    if !is_frozen {
        panic!("NotFrozen: account is not frozen");
    }

    // 3. Clear or invert state safely
    env.storage().instance().set(&account_id, &false);
}