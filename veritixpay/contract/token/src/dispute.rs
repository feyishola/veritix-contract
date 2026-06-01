//! Dispute resolution module.
//! Uses designated resolvers to settle contested escrows.

use crate::balance::{receive_balance, spend_balance};
use crate::escrow::get_escrow;
use crate::storage_types::{increment_counter, write_persistent_record, DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, symbol_short, vec, Address, Bytes, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Open,
    ResolvedForBeneficiary,
    ResolvedForDepositor,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub id: u32,
    pub escrow_id: u32,
    pub claimant: Address,
    pub resolver: Address,
    pub status: DisputeStatus,
    pub evidence: Bytes,
    pub opened_at_ledger: u32,
    pub expiry_ledger: u32,
}

fn bump_dispute(e: &Env, key: &DataKey) {
    e.storage().persistent().extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

fn append_dispute_history(e: &Env, escrow_id: u32, dispute_id: u32) {
    let key = DataKey::EscrowDisputeHistory(escrow_id);
    let mut ids: Vec<u32> = e.storage().persistent().get(&key).unwrap_or_else(|| vec![e]);
    ids.push_back(dispute_id);
    e.storage().persistent().set(&key, &ids);
    bump_dispute(e, &key);
}

fn append_open_dispute(e: &Env, id: u32) {
    let key = DataKey::OpenDisputes;
    let mut ids: Vec<u32> = e.storage().persistent().get(&key).unwrap_or_else(|| vec![e]);
    ids.push_back(id);
    e.storage().persistent().set(&key, &ids);
    bump_dispute(e, &key);
}

fn remove_open_dispute(e: &Env, id: u32) {
    let key = DataKey::OpenDisputes;
    let ids: Vec<u32> = e.storage().persistent().get(&key).unwrap_or_else(|| vec![e]);
    let mut updated: Vec<u32> = vec![e];
    for i in 0..ids.len() { let v = ids.get(i).unwrap(); if v != id { updated.push_back(v); } }
    e.storage().persistent().set(&key, &updated);
    bump_dispute(e, &key);
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

pub fn open_dispute(e: &Env, claimant: Address, escrow_id: u32, resolver: Address, evidence: Bytes, expiry_ledger: u32) -> u32 {
    claimant.require_auth();
    if evidence.len() > 128 { panic!("EvidenceTooLong: evidence cannot exceed 128 bytes"); }
    let current = e.ledger().sequence();
    if expiry_ledger <= current { panic!("InvalidExpiry: expiry_ledger must be in the future"); }
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
        status: DisputeStatus::Open, evidence: evidence.clone(),
        opened_at_ledger: current, expiry_ledger,
    };
    let dispute_key = DataKey::Dispute(count);
    e.storage().persistent().set(&dispute_key, &record);
    bump_dispute(e, &dispute_key);
    e.storage().persistent().set(&DataKey::EscrowDispute(escrow_id), &count);
    bump_dispute(e, &DataKey::EscrowDispute(escrow_id));
    append_dispute_history(e, escrow_id, count);
    append_open_dispute(e, count);
    e.events().publish((symbol_short!("disp_opened"), escrow_id, claimant.clone()), evidence);
    count
}

pub fn resolve_dispute(e: &Env, resolver: Address, dispute_id: u32, release_to_beneficiary: bool) {
    resolver.require_auth();
    let dispute_key = DataKey::Dispute(dispute_id);
    let mut dispute: DisputeRecord = e.storage().persistent().get(&dispute_key).expect("Dispute not found");
    bump_dispute(e, &dispute_key);
    if dispute.status != DisputeStatus::Open { panic!("AlreadyResolved: This dispute has already been resolved"); }
    if dispute.resolver != resolver { panic!("UnauthorizedResolver: Only the designated resolver can resolve this"); }
    settle_escrow_by_outcome(e, dispute.escrow_id, release_to_beneficiary);
    dispute.status = if release_to_beneficiary { DisputeStatus::ResolvedForBeneficiary } else { DisputeStatus::ResolvedForDepositor };
    e.storage().persistent().set(&dispute_key, &dispute);
    bump_dispute(e, &dispute_key);
    e.storage().persistent().remove(&DataKey::EscrowDispute(dispute.escrow_id));
    remove_open_dispute(e, dispute_id);
    e.events().publish((symbol_short!("disp_resolved"), dispute_id, resolver), release_to_beneficiary);
}

/// Anyone can call this after `expiry_ledger` has passed.
/// Auto-resolves in the depositor's favour if the resolver has not acted.
pub fn expire_dispute(e: &Env, dispute_id: u32) {
    let dispute_key = DataKey::Dispute(dispute_id);
    let mut dispute: DisputeRecord = e.storage().persistent().get(&dispute_key).expect("Dispute not found");
    bump_dispute(e, &dispute_key);
    if dispute.status != DisputeStatus::Open { panic!("AlreadyResolved: dispute is not open"); }
    if e.ledger().sequence() <= dispute.expiry_ledger { panic!("NotExpired: expiry ledger has not been reached"); }
    let escrow_id = dispute.escrow_id;
    settle_escrow_by_outcome(e, escrow_id, false);
    dispute.status = DisputeStatus::Expired;
    e.storage().persistent().set(&dispute_key, &dispute);
    bump_dispute(e, &dispute_key);
    e.storage().persistent().remove(&DataKey::EscrowDispute(escrow_id));
    remove_open_dispute(e, dispute_id);
    e.events().publish((symbol_short!("disp_expired"), dispute_id), escrow_id);
}

pub fn get_dispute(e: &Env, dispute_id: u32) -> DisputeRecord {
    let key = DataKey::Dispute(dispute_id);
    let record = e.storage().persistent().get(&key).expect("Dispute not found");
    bump_dispute(e, &key);
    record
}

pub fn get_dispute_history_for_escrow(e: &Env, escrow_id: u32) -> Vec<u32> {
    let key = DataKey::EscrowDisputeHistory(escrow_id);
    e.storage().persistent().get(&key).unwrap_or_else(|| vec![e])
}

pub fn get_open_disputes(e: &Env) -> Vec<u32> {
    let key = DataKey::OpenDisputes;
    e.storage().persistent().get(&key).unwrap_or_else(|| vec![e])
}
