use crate::storage_types::{
    AllowanceDataKey, AllowanceValue, DataKey, ALLOWANCE_BUMP_AMOUNT, ALLOWANCE_LIFETIME_THRESHOLD,
    PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD,
};
use crate::validation::{require_current_or_future_ledger, require_non_negative_amount};
use soroban_sdk::{Address, Env, Vec};

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
            e.storage().persistent().extend_ttl(
                &key,
                ALLOWANCE_LIFETIME_THRESHOLD,
                ALLOWANCE_BUMP_AMOUNT,
            );
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

    let index_key = DataKey::SpenderAllowances(spender.clone());
    let mut spenders_from: Vec<Address> = e
        .storage()
        .persistent()
        .get(&index_key)
        .unwrap_or_else(|| Vec::new(e));

    if amount == 0 {
        e.storage().persistent().remove(&key);
        let mut updated = Vec::new(e);
        for i in 0..spenders_from.len() {
            let addr = spenders_from.get(i).unwrap();
            if addr != from {
                updated.push_back(addr);
            }
        }
        e.storage().persistent().set(&index_key, &updated);
        // Keep spender index alive for long-lived delegated payment lookups.
        e.storage().persistent().extend_ttl(
            &index_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    } else {
        let mut exists = false;
        for i in 0..spenders_from.len() {
            if spenders_from.get(i).unwrap() == from {
                exists = true;
                break;
            }
        }
        if !exists {
            spenders_from.push_back(from.clone());
            e.storage().persistent().set(&index_key, &spenders_from);
            // Keep spender index alive for long-lived delegated payment lookups.
            e.storage().persistent().extend_ttl(
                &index_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }
        let allowance = AllowanceValue {
            amount,
            expiration_ledger,
        };
        e.storage().persistent().set(&key, &allowance);
        e.storage().persistent().extend_ttl(
            &key,
            ALLOWANCE_LIFETIME_THRESHOLD,
            ALLOWANCE_BUMP_AMOUNT,
        );
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

pub fn get_allowances_for_spender(e: &Env, spender: Address) -> Vec<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::SpenderAllowances(spender))
        .unwrap_or_else(|| Vec::new(e))
}

pub fn increase_allowance(e: &Env, from: Address, spender: Address, delta: i128, expiration_ledger: u32) {
    from.require_auth();
    let current = read_allowance(e, from.clone(), spender.clone());
    let new_amount = current.amount.checked_add(delta).expect("allowance overflow");
    write_allowance(e, from, spender, new_amount, expiration_ledger);
}

pub fn decrease_allowance(e: &Env, from: Address, spender: Address, delta: i128) {
    from.require_auth();
    let current = read_allowance(e, from.clone(), spender.clone());
    let new_amount = if delta > current.amount { 0 } else { current.amount - delta };
    write_allowance(e, from, spender, new_amount, current.expiration_ledger);
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
