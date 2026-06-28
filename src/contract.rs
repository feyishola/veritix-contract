use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};
use crate::{escrow, multi_escrow, allowance};
use crate::admin::validate_admin_address;
use crate::storage_types::DataKey;
use crate::validation::require_positive_amount; // Security audit import

pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initializes the global escrow contract configuration state
    ///
    /// # Arguments
    /// * `env` - The current execution environment.
    /// * `admin` - The primary controller keypair address.
    ///
    /// # Safety Warning
    /// **CRITICAL:** The `admin` address must be a funded, valid, and accessible account or 
    /// deployed contract signature context. Initializing the contract with an incorrect, dead, 
    /// or un-controlled address will permanently and irreversibly lock all admin-restricted actions.
    pub fn initialize(env: Env, admin: Address) {
        // 1. Strict Requirement: Fail-fast if the input address is unsafe or uninitialized
        validate_admin_address(&env, &admin);

        // 2. Assert initialization hasn't run yet
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("AlreadyInitialized: contract state is locked");
        }

        // 3. Save admin address securely to persistent state storage
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }
}

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
    fn get_escrows_batch(e: Env, escrow_ids: Vec<u32>) -> Vec<Option<escrow::EscrowRecord>>;
    fn get_escrow_age(e: Env, escrow_id: u32) -> u32;

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

    // ── Allowance ─────────────────────────────────────────────────────────────
    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);
    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);
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
        require_positive_amount(amount);
        escrow::create_escrow(e, depositor, beneficiary, token, amount, expiry_ledger, memo)
    }

    fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow::release_escrow(e, caller, escrow_id)
    }

    fn release_partial_escrow(e: Env, caller: Address, escrow_id: u32, amount: i128) {
        require_positive_amount(amount);
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

    fn get_escrows_batch(e: Env, escrow_ids: Vec<u32>) -> Vec<Option<escrow::EscrowRecord>> {
        escrow::get_escrows_batch(e, escrow_ids)
    }

    fn get_escrow_age(e: Env, escrow_id: u32) -> u32 {
        escrow::get_escrow_age(e, escrow_id)
    }

    fn create_multi_escrow(
        e: Env,
        depositor: Address,
        recipients: Vec<(Address, i128)>,
        token: Address,
        expiry_ledger: u32,
    ) -> u32 {
        // Enforce that total distributed amount values are checked within sub-module contexts
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
        require_positive_amount(ticket_price);
        
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
        require_positive_amount(total_amount);
        
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

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        require_positive_amount(amount);
        allowance::create_allowance(&e, &from, &spender, amount, expiration_ledger);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        require_positive_amount(amount);
        allowance::spend_allowance(&e, &from, &spender, amount);
        // Implement the actual token transfer logic here
    }
}