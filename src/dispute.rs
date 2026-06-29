use soroban_sdk::{Env, Address, Vec};
use crate::storage_types::DataKey;

pub fn open_dispute(e: Env, claimant: Address, escrow_id: u32, dispute_id: u32) {
    claimant.require_auth();
    let mut disputes = get_disputes_by_claimant(e.clone(), claimant.clone());
    disputes.push_back(dispute_id);
    e.storage().persistent().set(&DataKey::ClaimantDisputes(claimant), &disputes);
}

pub fn get_disputes_by_claimant(e: Env, claimant: Address) -> Vec<u32> {
    e.storage()
        .persistent()
        .get(&DataKey::ClaimantDisputes(claimant))
        .unwrap_or(Vec::new(&e))
}
