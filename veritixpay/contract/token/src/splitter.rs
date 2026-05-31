use crate::balance::{receive_balance, spend_balance};
use crate::storage_types::{
    increment_counter, read_persistent_record, write_persistent_record, DataKey,
    PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD,
};
use crate::validation::require_positive_amount;
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitRecipient {
    pub address: Address,
    pub share_bps: u32, // 10000 bps = 100%
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitRecord {
    pub id: u32,
    pub sender: Address,
    pub recipients: Vec<SplitRecipient>,
    pub total_amount: i128,
    pub distributed: bool,
    pub cancelled: bool,
    pub memo: Bytes,
}

pub fn create_split(
    e: &Env,
    sender: Address,
    recipients: Vec<SplitRecipient>,
    total_amount: i128,
) -> u32 {
    require_positive_amount(total_amount);
    sender.require_auth();

    // 1. Reject empty recipient list
    if recipients.is_empty() {
        panic!("recipients list cannot be empty");
    }

    // Cap at 20 recipients to stay within Soroban per-tx computational and
    // ledger entry limits. Exceeding this could cause mid-distribution failures
    // that leave funds stuck in the contract.
    if recipients.len() > 20 {
        panic!("TooManyRecipients: maximum 20 recipients allowed");
    }

    // 2. Validate recipients: no zero-share, no duplicates; BPS sums to 10000
    let mut total_bps: u32 = 0;
    for i in 0..recipients.len() {
        let r = recipients.get(i).unwrap();
        if r.share_bps == 0 {
            panic!("recipient share_bps cannot be zero");
        }
        for j in (i + 1)..recipients.len() {
            if r.address == recipients.get(j).unwrap().address {
                panic!("duplicate recipient address");
            }
        }
        total_bps += r.share_bps;
    }
    if total_bps != 10000 {
        panic!("total bps must equal 10000");
    }

    // 2. Increment and get Split ID
    let count = increment_counter(e, &DataKey::SplitCount);

    // 3. Move funds from sender to contract
    // Note: Assuming contract address is e.current_contract_address()
    spend_balance(e, sender.clone(), total_amount);
    receive_balance(e, e.current_contract_address(), total_amount);

    // 4. Store record
    let record = SplitRecord {
        id: count,
        sender,
        recipients,
        total_amount,
        distributed: false,
        cancelled: false,
        memo: Bytes::new(e),
    };
    write_persistent_record(e, &DataKey::Split(count), &record);

    count
}

pub fn distribute(e: &Env, caller: Address, split_id: u32) {
    caller.require_auth();

    let mut record: SplitRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Split(split_id))
        .expect("split record not found");
    e.storage().persistent().extend_ttl(
        &DataKey::Split(split_id),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    // 1. Rules: Caller must be sender, cannot distribute twice
    if record.sender != caller {
        panic!("unauthorized");
    }
    if record.distributed {
        panic!("already distributed");
    }
    if record.cancelled {
        panic!("split cancelled");
    }

    let sender_for_event = record.sender.clone();
    let amount_for_event = record.total_amount;

    let mut remaining_amount = record.total_amount;
    let len = record.recipients.len();

    // 2. Proportional Distribution
    for (i, recipient) in record.recipients.iter().enumerate() {
        let amount_to_send = if i == (len as usize - 1) {
            // Last recipient gets everything left to avoid rounding dust
            remaining_amount
        } else {
            record
                .total_amount
                .checked_mul(recipient.share_bps as i128)
                .expect("split amount overflow")
                / 10000
        };

        // Transfer from contract to recipient
        spend_balance(e, e.current_contract_address(), amount_to_send);
        receive_balance(e, recipient.address.clone(), amount_to_send);

        remaining_amount = remaining_amount
            .checked_sub(amount_to_send)
            .expect("split remaining underflow");
    }

    // 3. Mark distributed
    record.distributed = true;
    write_persistent_record(e, &DataKey::Split(split_id), &record);

    // 4. Emit Observability Event
    e.events().publish(
        (
            symbol_short!("split_distributed"),
            split_id,
            sender_for_event,
        ),
        amount_for_event,
    );
}

pub fn cancel_split(e: &Env, caller: Address, split_id: u32) {
    caller.require_auth();

    let mut record: SplitRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Split(split_id))
        .expect("split record not found");
    e.storage().persistent().extend_ttl(
        &DataKey::Split(split_id),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    if record.sender != caller {
        panic!("unauthorized");
    }
    if record.distributed {
        panic!("already distributed");
    }
    if record.cancelled {
        panic!("already cancelled");
    }

    spend_balance(e, e.current_contract_address(), record.total_amount);
    receive_balance(e, caller.clone(), record.total_amount);

    record.cancelled = true;
    write_persistent_record(e, &DataKey::Split(split_id), &record);

    e.events().publish(
        (symbol_short!("split_cancelled"), split_id, caller),
        record.total_amount,
    );
}

pub fn get_split(e: &Env, split_id: u32) -> SplitRecord {
    read_persistent_record(e, &DataKey::Split(split_id), "split record not found")
}

/// Creates a split tagged with a memo (max 64 bytes) for off-chain correlation.
pub fn create_split_with_memo(
    e: &Env,
    sender: Address,
    recipients: Vec<SplitRecipient>,
    total_amount: i128,
    memo: Bytes,
) -> u32 {
    if memo.len() > 64 {
        panic!("MemoTooLong: memo cannot exceed 64 bytes");
    }
    let id = create_split(e, sender, recipients, total_amount);
    let mut record: SplitRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Split(id))
        .expect("split not found");
    record.memo = memo;
    write_persistent_record(e, &DataKey::Split(id), &record);
    id
}
