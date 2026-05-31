use crate::balance::{receive_balance, spend_balance};
use crate::escrow::get_escrow;
use crate::storage_types::{increment_counter, write_persistent_record, DataKey, DISPUTE_BUMP_AMOUNT, DISPUTE_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use crate::storage_types::{increment_counter, write_persistent_record, DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Symbol};
use crate::storage_types::{
    increment_counter, write_persistent_record, DataKey, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_LIFETIME_THRESHOLD,
};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};
use soroban_sdk::{contracttype, symbol_short, vec, Address, Env, Symbol, Vec};

pub const APPEAL_WINDOW: u32 = 1000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus { Open, ResolvedForBeneficiary, ResolvedForDepositor, Appealed }
pub enum DisputeStatus { Open, ResolvedForBeneficiary, ResolvedForDepositor }

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub id: u32,
    pub escrow_id: u32,
    pub claimant: Address,
    pub resolver: Address,
    pub status: DisputeStatus,
    pub resolution_note: Bytes,
    pub id: u32, pub escrow_id: u32, pub claimant: Address,
    pub resolver: Address, pub status: DisputeStatus,
    pub appeal_deadline_ledger: u32,
}

fn append_dispute_history(e: &Env, escrow_id: u32, dispute_id: u32) {
    let key = DataKey::EscrowDisputeHistory(escrow_id);
    let mut ids: Vec<u32> = e.storage().persistent().get(&key).unwrap_or_else(|| vec![e]);
    ids.push_back(dispute_id);
    e.storage().persistent().set(&key, &ids);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

fn append_resolver_dispute(e: &Env, resolver: &Address, id: u32) {
    let key = DataKey::ResolverDisputes(resolver.clone());
fn append_open_dispute(e: &Env, id: u32) {
    let key = DataKey::OpenDisputes;
    let mut ids: Vec<u32> = e.storage().persistent().get(&key).unwrap_or_else(|| vec![e]);
    ids.push_back(id);
    e.storage().persistent().set(&key, &ids);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

fn remove_resolver_dispute(e: &Env, resolver: &Address, id: u32) {
    let key = DataKey::ResolverDisputes(resolver.clone());
fn remove_open_dispute(e: &Env, id: u32) {
    let key = DataKey::OpenDisputes;
    let ids: Vec<u32> = e.storage().persistent().get(&key).unwrap_or_else(|| vec![e]);
    let mut updated: Vec<u32> = vec![e];
    for i in 0..ids.len() {
        let v = ids.get(i).unwrap();
        if v != id { updated.push_back(v); }
    }
    e.storage().persistent().set(&key, &updated);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn open_dispute(e: &Env, claimant: Address, escrow_id: u32, resolver: Address) -> u32 {
    claimant.require_auth();
    let escrow = get_escrow(e, escrow_id);
    if escrow.released || escrow.refunded { panic!("InvalidState: Cannot open dispute on a settled escrow"); }
    if claimant != escrow.depositor && claimant != escrow.beneficiary { panic!("Unauthorized: Only depositor or beneficiary can open a dispute"); }
    if resolver == claimant { panic!("InvalidResolver: resolver cannot be the claimant"); }
    if resolver == escrow.depositor { panic!("InvalidResolver: resolver cannot be the depositor"); }
    if resolver == escrow.beneficiary { panic!("InvalidResolver: resolver cannot be the beneficiary"); }
    if e.storage().persistent().has(&DataKey::EscrowDispute(escrow_id)) { panic!("DisputeAlreadyOpen: An open dispute already exists for this escrow"); }
    let count = increment_counter(e, &DataKey::DisputeCount);
    let record = DisputeRecord {
        id: count, escrow_id, claimant: claimant.clone(), resolver,
        status: DisputeStatus::Open, appeal_deadline_ledger: 0,
    let count = increment_counter(e, &DataKey::DisputeCount);
    let record = DisputeRecord { id: count, escrow_id, claimant: claimant.clone(), resolver, status: DisputeStatus::Open };
        if v != id {
            updated.push_back(v);
        }
    }
    e.storage().persistent().set(&key, &updated);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn open_dispute(e: &Env, claimant: Address, escrow_id: u32, resolver: Address) -> u32 {
    claimant.require_auth();
    let escrow = get_escrow(e, escrow_id);
    if escrow.released || escrow.refunded {
        panic!("InvalidState: Cannot open dispute on a settled escrow");
    }
    if claimant != escrow.depositor && claimant != escrow.beneficiary {
        panic!("Unauthorized: Only depositor or beneficiary can open a dispute");
    }
    if resolver == claimant { panic!("InvalidResolver: resolver cannot be the claimant"); }
    if resolver == escrow.depositor { panic!("InvalidResolver: resolver cannot be the depositor"); }
    if resolver == escrow.beneficiary { panic!("InvalidResolver: resolver cannot be the beneficiary"); }
    if e.storage().persistent().has(&DataKey::EscrowDispute(escrow_id)) {
        panic!("DisputeAlreadyOpen: An open dispute already exists for this escrow");
    }
    let count = increment_counter(e, &DataKey::DisputeCount);
    let record = DisputeRecord {
        id: count,
        escrow_id,
        claimant: claimant.clone(),
        resolver,
        status: DisputeStatus::Open,
        resolution_note: Bytes::new(e),
        id: count, escrow_id, claimant: claimant.clone(), resolver: resolver.clone(), status: DisputeStatus::Open,
        id: count, escrow_id, claimant: claimant.clone(), resolver, status: DisputeStatus::Open,
    };
    let dispute_key = DataKey::Dispute(count);
    let escrow_dispute_key = DataKey::EscrowDispute(escrow_id);
    e.storage().persistent().set(&dispute_key, &record);
    e.storage()
        .persistent()
        .extend_ttl(&dispute_key, DISPUTE_LIFETIME_THRESHOLD, DISPUTE_BUMP_AMOUNT);
    e.storage().persistent().extend_ttl(&dispute_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.storage().persistent().set(&escrow_dispute_key, &count);
    e.storage().persistent().extend_ttl(&escrow_dispute_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    append_dispute_history(e, escrow_id, count);
    append_resolver_dispute(e, &resolver, count);
    append_open_dispute(e, count);
    e.events().publish((symbol_short!("dispute_opened"), escrow_id, claimant.clone()), ());
    count
}

fn settle_escrow_by_outcome(e: &Env, escrow_id: u32, release_to_beneficiary: bool) {
    let mut escrow = get_escrow(e, escrow_id);
    if escrow.released || escrow.refunded { panic!("AlreadySettled: escrow is already settled"); }
    if release_to_beneficiary {
        escrow.released = true;
        write_persistent_record(e, &DataKey::Escrow(escrow_id), &escrow);
        spend_balance(e, e.current_contract_address(), escrow.amount);
        receive_balance(e, escrow.beneficiary.clone(), escrow.amount);
        e.events().publish((symbol_short!("escrow_released"), escrow_id, escrow.beneficiary.clone()), escrow.amount);
    } else {
        escrow.refunded = true;
        write_persistent_record(e, &DataKey::Escrow(escrow_id), &escrow);
        spend_balance(e, e.current_contract_address(), escrow.amount);
        receive_balance(e, escrow.depositor.clone(), escrow.amount);
        e.events().publish((symbol_short!("escrow_refunded"), escrow_id, escrow.depositor.clone()), escrow.amount);
    }
}

pub fn resolve_dispute(e: &Env, resolver: Address, dispute_id: u32, release_to_beneficiary: bool) {
    resolver.require_auth();
    let dispute_key = DataKey::Dispute(dispute_id);
    let mut dispute: DisputeRecord = e
        .storage()
        .persistent()
        .get(&dispute_key)
        .expect("Dispute not found");
    e.storage()
        .persistent()
        .extend_ttl(&dispute_key, DISPUTE_LIFETIME_THRESHOLD, DISPUTE_BUMP_AMOUNT);

    if dispute.status != DisputeStatus::Open {
        panic!("AlreadyResolved: This dispute has already been resolved");
    }

    if dispute.resolver != resolver {
        panic!("UnauthorizedResolver: Only the designated resolver can resolve this");
    }

    let mut dispute: DisputeRecord = e.storage().persistent().get(&dispute_key).expect("Dispute not found");
    e.storage().persistent().extend_ttl(&dispute_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    if dispute.status != DisputeStatus::Open { panic!("AlreadyResolved: This dispute has already been resolved"); }
    if dispute.resolver != resolver { panic!("UnauthorizedResolver: Only the designated resolver can resolve this"); }
    settle_escrow_by_outcome(e, dispute.escrow_id, release_to_beneficiary);
    dispute.status = if release_to_beneficiary { DisputeStatus::ResolvedForBeneficiary } else { DisputeStatus::ResolvedForDepositor };
    dispute.appeal_deadline_ledger = e.ledger().sequence() + APPEAL_WINDOW;
    e.storage().persistent().set(&dispute_key, &dispute);
    e.storage().persistent().extend_ttl(&dispute_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.storage().persistent().remove(&DataKey::EscrowDispute(dispute.escrow_id));
    e.events().publish((symbol_short!("dispute_resolved"), dispute_id, resolver), release_to_beneficiary);
}

pub fn appeal_dispute(e: &Env, appellant: Address, dispute_id: u32, new_resolver: Address) {
    appellant.require_auth();
    let dispute_key = DataKey::Dispute(dispute_id);
    let mut dispute: DisputeRecord = e.storage().persistent().get(&dispute_key).expect("Dispute not found");
    if dispute.status == DisputeStatus::Open || dispute.status == DisputeStatus::Appealed {
        panic!("InvalidState: dispute must be resolved before appeal");
    }
    if e.ledger().sequence() > dispute.appeal_deadline_ledger { panic!("AppealWindowClosed: appeal window has passed"); }
    if appellant != dispute.claimant { panic!("Unauthorized: only the claimant can appeal"); }
    dispute.status = DisputeStatus::Appealed;
    dispute.resolver = new_resolver.clone();
    e.storage().persistent().set(&dispute_key, &dispute);
    e.storage().persistent().extend_ttl(&dispute_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("dispute_appealed"), dispute_id, appellant), new_resolver);
    remove_resolver_dispute(e, &resolver, dispute_id);
    dispute.status = if release_to_beneficiary { DisputeStatus::ResolvedForBeneficiary } else { DisputeStatus::ResolvedForDepositor };
    e.storage().persistent().set(&dispute_key, &dispute);
    e.storage().persistent().extend_ttl(&dispute_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.storage().persistent().remove(&DataKey::EscrowDispute(dispute.escrow_id));
    remove_open_dispute(e, dispute_id);
    e.events().publish((symbol_short!("dispute_resolved"), dispute_id, resolver), release_to_beneficiary);
}

pub fn get_dispute(e: &Env, dispute_id: u32) -> DisputeRecord {
    let key = DataKey::Dispute(dispute_id);
    let record = e.storage().persistent().get(&key).expect("Dispute not found");
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    record
}

/// Resolves a dispute and attaches a permanent resolution note (max 128 bytes).
pub fn resolve_dispute_with_note(
    e: &Env,
    resolver: Address,
    dispute_id: u32,
    release_to_beneficiary: bool,
    note: Bytes,
) {
    if note.len() > 128 {
        panic!("NoteTooLong: resolution note cannot exceed 128 bytes");
    }
    resolve_dispute(e, resolver, dispute_id, release_to_beneficiary);
    let dispute_key = DataKey::Dispute(dispute_id);
    let mut record: DisputeRecord = e
        .storage()
        .persistent()
        .get(&dispute_key)
        .expect("Dispute not found");
    record.resolution_note = note;
    e.storage()
        .persistent()
        .extend_ttl(&key, DISPUTE_LIFETIME_THRESHOLD, DISPUTE_BUMP_AMOUNT);
    record
        .set(&dispute_key, &record);
    e.storage()
        .persistent()
        .extend_ttl(&dispute_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}
pub fn get_dispute_history_for_escrow(e: &Env, escrow_id: u32) -> Vec<u32> {
    let key = DataKey::EscrowDisputeHistory(escrow_id);
pub fn get_disputes_by_resolver(e: &Env, resolver: Address) -> Vec<u32> {
    let key = DataKey::ResolverDisputes(resolver);
pub fn get_open_disputes(e: &Env) -> Vec<u32> {
    let key = DataKey::OpenDisputes;
    e.storage().persistent().get(&key).unwrap_or_else(|| vec![e])
}
