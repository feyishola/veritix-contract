use crate::storage_types::{
    AllowanceDataKey, AllowanceValue, DataKey, ALLOWANCE_BUMP_AMOUNT, ALLOWANCE_LIFETIME_THRESHOLD,
};
use crate::validation::{require_current_or_future_ledger, require_non_negative_amount};
use soroban_sdk::{Address, Env};

pub fn read_allowance(e: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });

    if let Some(allowance) = e
        .storage()
        .persistent()
        .get::<DataKey, AllowanceValue>(&key)
    {
        // Equal-to-current-ledger approvals are still valid for the current ledger.
        // They become expired only once the sequence advances past expiration_ledger.
        if allowance.expiration_ledger < e.ledger().sequence() {
            // Prune expired entry from storage
            e.storage().persistent().remove(&key);
            AllowanceValue {
                amount: 0,
                expiration_ledger: allowance.expiration_ledger,
            }
        } else {
            // Extend TTL on active allowance read
            e.storage()
                .persistent()
                .extend_ttl(&key, ALLOWANCE_LIFETIME_THRESHOLD, ALLOWANCE_BUMP_AMOUNT);
            allowance
        }
    } else {
        AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }
    }
}

pub fn write_allowance(
    e: &Env,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) {
    require_non_negative_amount(amount);
    require_current_or_future_ledger(e.ledger().sequence(), expiration_ledger);

    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });

    if amount == 0 {
        e.storage().persistent().remove(&key);
    } else {
        let allowance = AllowanceValue {
            amount,
            expiration_ledger,
        };
        e.storage().persistent().set(&key, &allowance);
    }
}

/// Check that the allowance exists, is non-expired, and covers `amount` WITHOUT spending it.
/// Call this before `require_auth()` so a definitely-failing call never emits an auth event.
pub fn validate_allowance(e: &Env, from: Address, spender: Address, amount: i128) {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    let allowance = e
        .storage()
        .persistent()
        .get::<DataKey, AllowanceValue>(&key)
        .unwrap_or(AllowanceValue { amount: 0, expiration_ledger: 0 });
    if allowance.expiration_ledger < e.ledger().sequence() {
        panic!("allowance is expired");
    }
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
}

pub fn spend_allowance(e: &Env, from: Address, spender: Address, amount: i128) {
    let allowance = read_allowance(e, from.clone(), spender.clone());

    // Spending is allowed when expiration_ledger == current ledger sequence.
    if allowance.expiration_ledger < e.ledger().sequence() {
        panic!("allowance is expired");
    }

    if allowance.amount < amount {
        panic!("insufficient allowance");
    }

    write_allowance(
        e,
        from,
        spender,
        allowance.amount - amount,
        allowance.expiration_ledger,
    );
}
