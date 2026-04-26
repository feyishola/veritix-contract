use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use crate::escrow;
use crate::multi_escrow;

pub trait VeriTixPayTrait {
    fn create_escrow(
        e: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        expiry_ledger: u32,
    ) -> u32;

    fn release_escrow(e: Env, caller: Address, escrow_id: u32);
    fn refund_escrow(e: Env, caller: Address, escrow_id: u32);
    fn get_escrows_by_depositor(e: Env, depositor: Address) -> Vec<u32>;
    fn get_escrows_by_beneficiary(e: Env, beneficiary: Address) -> Vec<u32>; // NEW

    // multi-escrow — see #176
    fn create_multi_escrow(
        e: Env,
        depositor: Address,
        recipients: Vec<(Address, i128)>,
        token: Address,
        expiry_ledger: u32,
    ) -> u32;
    fn release_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32);
    fn refund_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32);
}

#[contract]
pub struct VeriTixPay;

#[contractimpl]
impl VeriTixPayTrait for VeriTixPay {
    fn create_escrow(
        e: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        expiry_ledger: u32,
    ) -> u32 {
        escrow::create_escrow(e, depositor, beneficiary, token, amount, expiry_ledger)
    }

    fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow::release_escrow(e, caller, escrow_id)
    }

    fn refund_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow::refund_escrow(e, caller, escrow_id)
    }

    fn get_escrows_by_depositor(e: Env, depositor: Address) -> Vec<u32> {
        escrow::get_escrows_by_depositor(e, depositor)
    }

    fn get_escrows_by_beneficiary(e: Env, beneficiary: Address) -> Vec<u32> {
        escrow::get_escrows_by_beneficiary(e, beneficiary)
    }

    fn create_multi_escrow(
        e: Env,
        depositor: Address,
        recipients: Vec<(Address, i128)>,
        token: Address,
        expiry_ledger: u32,
    ) -> u32 {
        multi_escrow::create_multi_escrow(e, depositor, recipients, token, expiry_ledger)
    }

    fn release_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32) {
        multi_escrow::release_multi_escrow(e, caller, multi_escrow_id)
    }

    fn refund_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32) {
        multi_escrow::refund_multi_escrow(e, caller, multi_escrow_id)
    }
}
