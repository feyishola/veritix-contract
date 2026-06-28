#![cfg(test)]

use soroban_sdk::{Env, testutils::Address as _};
use crate::dispute::{open_dispute, get_disputes_by_claimant};

#[test]
fn test_open_dispute_adds_to_claimant_history() {
    let e = Env::default();
    e.mock_all_auths();

    let claimant = soroban_sdk::Address::generate(&e);
    let escrow_id = 1;
    let dispute_id = 100;
    
    open_dispute(e.clone(), claimant.clone(), escrow_id, dispute_id);
    
    let history = get_disputes_by_claimant(e.clone(), claimant.clone());
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap(), dispute_id);
    
    let dispute_id2 = 101;
    open_dispute(e.clone(), claimant.clone(), escrow_id, dispute_id2);
    let history = get_disputes_by_claimant(e.clone(), claimant.clone());
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(1).unwrap(), dispute_id2);
}
