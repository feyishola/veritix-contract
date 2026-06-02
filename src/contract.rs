use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};
use crate::{escrow, multi_escrow};

pub trait VeriTixPayTrait {
    // ── Escrow ────────────────────────────────────────────────────────────────
    fn create_escrow(
        e: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        expiry_ledger: u32,
        memo: Bytes,            // #175
    ) -> u32;

    fn release_escrow(e: Env, caller: Address, escrow_id: u32);
    fn release_partial_escrow(e: Env, caller: Address, escrow_id: u32, amount: i128); // #174
    fn refund_escrow(e: Env, caller: Address, escrow_id: u32);
    fn get_escrows_by_depositor(e: Env, depositor: Address) -> Vec<u32>;
    fn get_escrows_by_beneficiary(e: Env, beneficiary: Address) -> Vec<u32>;
    fn escrowed_total(e: Env) -> i128;

    // ── Multi-escrow ──────────────────────────────────────────────────────────
    fn create_multi_escrow(
        e: Env,
        depositor: Address,
        recipients: Vec<(Address, i128)>,
        token: Address,
        expiry_ledger: u32,
    ) -> u32;
    fn release_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32);
    fn refund_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32);
    fn ticket_escrow(
        e: Env,
        buyer: Address,
        organizer: Address,
        token: Address,
        ticket_price: i128,
        event_ledger: u32,
        ticket_ref: Bytes,
    ) -> u32;
    fn revenue_split(
        e: Env,
        sender: Address,
        organizer: Address,
        organizer_bps: u32,
        artist: Address,
        artist_bps: u32,
        platform: Address,
        token: Address,
        total_amount: i128,
        event_ledger: u32,
    ) -> u32;
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
        memo: Bytes,
    ) -> u32 {
        escrow::create_escrow(e, depositor, beneficiary, token, amount, expiry_ledger, memo)
    }

    fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow::release_escrow(e, caller, escrow_id)
    }

    fn release_partial_escrow(e: Env, caller: Address, escrow_id: u32, amount: i128) {
        escrow::release_partial_escrow(e, caller, escrow_id, amount)
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

    fn escrowed_total(e: Env) -> i128 {
        escrow::get_escrowed_total(&e)
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

    fn ticket_escrow(
        e: Env,
        buyer: Address,
        organizer: Address,
        token: Address,
        ticket_price: i128,
        event_ledger: u32,
        ticket_ref: Bytes,
    ) -> u32 {
        buyer.require_auth();
        escrow::create_escrow(
            e,
            buyer,
            organizer,
            token,
            ticket_price,
            event_ledger + 100,
            ticket_ref,
        )
    }

    fn revenue_split(
        e: Env,
        sender: Address,
        organizer: Address,
        organizer_bps: u32,
        artist: Address,
        artist_bps: u32,
        platform: Address,
        token: Address,
        total_amount: i128,
        event_ledger: u32,
    ) -> u32 {
        sender.require_auth();
        assert!(organizer_bps + artist_bps < 10_000, "invalid basis points");
        let platform_bps = 10_000 - organizer_bps - artist_bps;
        let organizer_amt = total_amount * organizer_bps as i128 / 10_000;
        let artist_amt = total_amount * artist_bps as i128 / 10_000;
        let platform_amt = total_amount - organizer_amt - artist_amt;

        let recipients = Vec::from_array(
            &e,
            [
                (organizer, organizer_amt),
                (artist, artist_amt),
                (platform, platform_amt),
            ],
        );
        let split_id = multi_escrow::create_multi_escrow(
            e.clone(),
            sender.clone(),
            recipients,
            token,
            event_ledger + 100,
        );
        multi_escrow::release_multi_escrow(e, sender, split_id);
        split_id
    }
}
