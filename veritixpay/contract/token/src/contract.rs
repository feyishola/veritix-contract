use crate::admin::{check_admin, has_admin, read_admin, transfer_admin, write_admin};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{
    decrease_supply, increase_supply, read_balance, read_total_supply, receive_balance,
    spend_balance,
};
use crate::dispute::{
    get_dispute as dispute_get, get_disputes_by_resolver, open_dispute, resolve_dispute,
    DisputeRecord,
    get_dispute as dispute_get, get_open_disputes, open_dispute, resolve_dispute, DisputeRecord,
};
use crate::escrow::{
    admin_settle_escrow as escrow_admin_settle, create_escrow as escrow_create,
    get_escrow as escrow_get, refund_escrow as escrow_refund, release_escrow as escrow_release,
    EscrowRecord,
};
use crate::freeze::{freeze_account, is_frozen as read_frozen_status, unfreeze_account};
use crate::metadata::{
    read_decimal, read_name, read_symbol, validate_metadata, write_metadata, TokenMetadata,
};
use crate::recurring::{
    amend_recurring, cancel_recurring, execute_recurring, get_recurring, setup_recurring,
    RecurringRecord,
    cancel_recurring, execute_recurring, get_recurring, get_recurring_by_payer, setup_recurring,
    RecurringRecord,
    cancel_recurring, execute_recurring, get_recurring, pause_recurring, resume_recurring,
    setup_recurring, RecurringRecord,
};
use crate::splitter::{
    cancel_split as split_cancel, create_split as split_create, distribute as split_distribute,
    get_split as split_get, SplitRecord, SplitRecipient,
};
use crate::validation::{require_not_frozen_account, require_positive_amount};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Vec};

#[contract]
pub struct VeritixToken;

#[contractimpl]
impl VeritixToken {
    pub fn initialize(e: Env, admin: Address, name: String, symbol: String, decimal: u32) {
        if has_admin(&e) { panic!("already initialized"); }
    pub fn initialize(e: Env, admin: Address, name: String, symbol: String, decimal: u32) {
        if has_admin(&e) { panic!("already initialized"); }
    pub fn initialize(e: Env, admin: Address, name: String, symbol: String, decimal: u32) {
        if has_admin(&e) { panic!("already initialized"); }
    pub fn initialize(e: Env, admin: Address, name: String, symbol: String, decimal: u32) {
        if has_admin(&e) { panic!("already initialized"); }
    // --- Admin & metadata ---

    pub fn initialize(e: Env, admin: Address, name: String, symbol: String, decimal: u32) {
        if has_admin(&e) {
            panic!("already initialized");
        }
        admin.require_auth();
        let meta = TokenMetadata { name, symbol, decimal };
        validate_metadata(&meta);
        write_admin(&e, &admin);
        write_metadata(&e, meta);
    }
    pub fn set_admin(e: Env, new_admin: Address) { transfer_admin(&e, new_admin); }
    pub fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_admin(&e, &admin); require_positive_amount(amount);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
        e.events().publish((symbol_short!("clawback"), admin, from), amount);
    }

    pub fn set_admin(e: Env, new_admin: Address) { transfer_admin(&e, new_admin); }

    pub fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_admin(&e, &admin); require_positive_amount(amount);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);

    pub fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_admin(&e, &admin); require_positive_amount(amount);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
    pub fn set_admin(e: Env, new_admin: Address) {
        transfer_admin(&e, new_admin);
    }

    pub fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_admin(&e, &admin);
        require_positive_amount(amount);
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);
        e.events().publish((symbol_short!("clawback"), admin, from), amount);
    }

    pub fn freeze(e: Env, target: Address) {
        let admin = read_admin(&e); check_admin(&e, &admin); freeze_account(&e, admin, target);
    }
    pub fn unfreeze(e: Env, target: Address) {
        let admin = read_admin(&e); check_admin(&e, &admin); unfreeze_account(&e, admin, target);
    }
    pub fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_admin(&e, &admin); require_positive_amount(amount); require_not_frozen_account(&e, &to);
        receive_balance(&e, to.clone(), amount); increase_supply(&e, amount);
        e.events().publish((symbol_short!("mint"), admin, to), amount);
    }

    pub fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_admin(&e, &admin); require_positive_amount(amount); require_not_frozen_account(&e, &to);
        receive_balance(&e, to.clone(), amount); increase_supply(&e, amount);
    pub fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_admin(&e, &admin); require_positive_amount(amount); require_not_frozen_account(&e, &to);
        receive_balance(&e, to.clone(), amount); increase_supply(&e, amount);
    pub fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_admin(&e, &admin); require_positive_amount(amount); require_not_frozen_account(&e, &to);
        receive_balance(&e, to.clone(), amount); increase_supply(&e, amount);
    // --- Token interface ---

    pub fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_admin(&e, &admin);
        require_positive_amount(amount);
        require_not_frozen_account(&e, &to);
        receive_balance(&e, to.clone(), amount);
        increase_supply(&e, amount);
        e.events().publish((symbol_short!("mint"), admin, to), amount);
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth(); require_positive_amount(amount);
        require_not_frozen_account(&e, &from); require_not_frozen_account(&e, &to);
        spend_balance(&e, from.clone(), amount); receive_balance(&e, to.clone(), amount);
        e.events().publish((symbol_short!("transfer"), from, to), amount);
    }
    pub fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth(); require_positive_amount(amount);
        require_not_frozen_account(&e, &from); require_not_frozen_account(&e, &to);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); receive_balance(&e, to.clone(), amount);
        e.events().publish((symbol_short!("transfer"), from, to), amount);
    }
    pub fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth(); require_positive_amount(amount); require_not_frozen_account(&e, &from);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
        e.events().publish((symbol_short!("burn"), from), amount);
    }
    pub fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth(); require_positive_amount(amount); require_not_frozen_account(&e, &from);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
        e.events().publish((symbol_short!("burn"), from), amount);
    }
    pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth(); require_positive_amount(amount);
        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        e.events().publish((symbol_short!("approve"), from, spender), amount);
    }
        from.require_auth();
        require_positive_amount(amount);
        require_not_frozen_account(&e, &from);
        require_not_frozen_account(&e, &to);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        e.events().publish((symbol_short!("transfer"), from, to), amount);
    }

    pub fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth(); require_positive_amount(amount);
        require_not_frozen_account(&e, &from); require_not_frozen_account(&e, &to);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); receive_balance(&e, to.clone(), amount);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); receive_balance(&e, to.clone(), amount);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); receive_balance(&e, to.clone(), amount);
        spender.require_auth();
        require_positive_amount(amount);
        require_not_frozen_account(&e, &from);
        require_not_frozen_account(&e, &to);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        e.events().publish((symbol_short!("transfer"), from, to), amount);
    }

    pub fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth(); require_positive_amount(amount); require_not_frozen_account(&e, &from);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
        from.require_auth();
        require_positive_amount(amount);
        require_not_frozen_account(&e, &from);
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);
        e.events().publish((symbol_short!("burn"), from), amount);
    }

    pub fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth(); require_positive_amount(amount); require_not_frozen_account(&e, &from);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount); decrease_supply(&e, amount);
        spender.require_auth();
        require_positive_amount(amount);
        require_not_frozen_account(&e, &from);
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);
        e.events().publish((symbol_short!("burn"), from), amount);
    }

    pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth(); require_positive_amount(amount);
        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        e.events().publish((symbol_short!("approve"), from, spender), amount);
        from.require_auth();
        require_positive_amount(amount);
        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        e.events().publish((symbol_short!("approve"), from, spender), amount);
    }

    // --- Read-only views ---

    pub fn total_supply(e: Env) -> i128 {
        read_total_supply(&e)
    }

    pub fn total_supply(e: Env) -> i128 { read_total_supply(&e) }
    pub fn balance(e: Env, id: Address) -> i128 { read_balance(&e, id) }
    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 { read_allowance(&e, from, spender).amount }
    pub fn admin(e: Env) -> Address { read_admin(&e) }
    pub fn is_frozen(e: Env, id: Address) -> bool { read_frozen_status(&e, &id) }
    pub fn decimals(e: Env) -> u32 { read_decimal(&e) }
    pub fn name(e: Env) -> String { read_name(&e) }
    pub fn symbol(e: Env) -> String { read_symbol(&e) }

    pub fn create_escrow(e: Env, depositor: Address, beneficiary: Address, amount: i128, expiry_ledger: u32) -> u32 {
        escrow_create(&e, depositor, beneficiary, amount, expiry_ledger)
    }
    pub fn release_escrow(e: Env, caller: Address, escrow_id: u32) { escrow_release(&e, caller, escrow_id) }
    pub fn refund_escrow(e: Env, caller: Address, escrow_id: u32) { escrow_refund(&e, caller, escrow_id) }
    pub fn get_escrow(e: Env, escrow_id: u32) -> EscrowRecord { escrow_get(&e, escrow_id) }

    pub fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow_release(&e, caller, escrow_id)
    }

    pub fn refund_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow_refund(&e, caller, escrow_id)
    }

    pub fn get_escrow(e: Env, escrow_id: u32) -> EscrowRecord {
        escrow_get(&e, escrow_id)
    }

    pub fn escrow_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::EscrowCount)
    }
    pub fn admin_settle_escrow(e: Env, admin: Address, escrow_id: u32, recipient: Address) {
        escrow_admin_settle(&e, admin, escrow_id, recipient)
    }

    pub fn admin_settle_escrow(e: Env, admin: Address, escrow_id: u32, recipient: Address) {
        escrow_admin_settle(&e, admin, escrow_id, recipient)
    }

    pub fn open_dispute(e: Env, claimant: Address, escrow_id: u32, resolver: Address) -> u32 {
        open_dispute(&e, claimant, escrow_id, resolver)
    }
    pub fn resolve_dispute(e: Env, resolver: Address, dispute_id: u32, release_to_beneficiary: bool) {
        resolve_dispute(&e, resolver, dispute_id, release_to_beneficiary)
    }
    pub fn get_dispute(e: Env, dispute_id: u32) -> DisputeRecord { dispute_get(&e, dispute_id) }
        resolve_dispute(&e, resolver, dispute_id, release_to_beneficiary)
    }
    pub fn get_dispute(e: Env, dispute_id: u32) -> DisputeRecord { dispute_get(&e, dispute_id) }
        resolve_dispute(&e, resolver, dispute_id, release_to_beneficiary)
    }
    pub fn get_dispute(e: Env, dispute_id: u32) -> DisputeRecord { dispute_get(&e, dispute_id) }
        resolve_dispute(&e, resolver, dispute_id, release_to_beneficiary)
    }
    pub fn get_dispute(e: Env, dispute_id: u32) -> DisputeRecord { dispute_get(&e, dispute_id) }

    pub fn resolve_dispute(e: Env, resolver: Address, dispute_id: u32, release_to_beneficiary: bool) {
        resolve_dispute(&e, resolver, dispute_id, release_to_beneficiary)
    }

    pub fn get_dispute(e: Env, dispute_id: u32) -> DisputeRecord {
        dispute_get(&e, dispute_id)
    }

    pub fn dispute_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::DisputeCount)
    }
    pub fn create_split(e: Env, sender: Address, recipients: Vec<SplitRecipient>, total_amount: i128) -> u32 {
        split_create(&e, sender, recipients, total_amount)
    }
    pub fn distribute(e: Env, caller: Address, split_id: u32) { split_distribute(&e, caller, split_id) }
    pub fn cancel_split(e: Env, caller: Address, split_id: u32) { split_cancel(&e, caller, split_id) }
    pub fn get_split(e: Env, split_id: u32) -> SplitRecord { split_get(&e, split_id) }
    pub fn get_disputes_by_resolver(e: Env, resolver: Address) -> Vec<u32> {
        get_disputes_by_resolver(&e, resolver)
    pub fn get_open_disputes(e: Env) -> Vec<u32> { get_open_disputes(&e) }

    pub fn create_split(e: Env, sender: Address, recipients: Vec<SplitRecipient>, total_amount: i128) -> u32 {
        split_create(&e, sender, recipients, total_amount)
    }
    pub fn distribute(e: Env, caller: Address, split_id: u32) { split_distribute(&e, caller, split_id) }
    pub fn cancel_split(e: Env, caller: Address, split_id: u32) { split_cancel(&e, caller, split_id) }
    pub fn get_split(e: Env, split_id: u32) -> SplitRecord { split_get(&e, split_id) }
        split_create(&e, sender, recipients, total_amount)
    }
    pub fn distribute(e: Env, caller: Address, split_id: u32) { split_distribute(&e, caller, split_id) }
    pub fn cancel_split(e: Env, caller: Address, split_id: u32) { split_cancel(&e, caller, split_id) }
    pub fn get_split(e: Env, split_id: u32) -> SplitRecord { split_get(&e, split_id) }
    // --- Splitter ---

    pub fn create_split(e: Env, sender: Address, recipients: Vec<SplitRecipient>, total_amount: i128) -> u32 {
        split_create(&e, sender, recipients, total_amount)
    }

    pub fn distribute(e: Env, caller: Address, split_id: u32) {
        split_distribute(&e, caller, split_id)
    }

    pub fn create_split(e: Env, sender: Address, recipients: Vec<SplitRecipient>, total_amount: i128) -> u32 {
        split_create(&e, sender, recipients, total_amount)
    }
    pub fn distribute(e: Env, caller: Address, split_id: u32) { split_distribute(&e, caller, split_id) }
    pub fn cancel_split(e: Env, caller: Address, split_id: u32) { split_cancel(&e, caller, split_id) }
    pub fn get_split(e: Env, split_id: u32) -> SplitRecord { split_get(&e, split_id) }

    pub fn split_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::SplitCount)
    }
    pub fn setup_recurring(e: Env, payer: Address, payee: Address, amount: i128, interval: u32) -> u32 {
        setup_recurring(&e, payer, payee, amount, interval)
    }
    pub fn execute_recurring(e: Env, recurring_id: u32) { execute_recurring(&e, recurring_id) }
    pub fn cancel_recurring(e: Env, caller: Address, recurring_id: u32) { cancel_recurring(&e, caller, recurring_id) }
    pub fn get_recurring(e: Env, recurring_id: u32) -> RecurringRecord { get_recurring(&e, recurring_id) }

    pub fn setup_recurring(e: Env, payer: Address, payee: Address, amount: i128, interval: u32) -> u32 {
        setup_recurring(&e, payer, payee, amount, interval)
    }
    pub fn execute_recurring(e: Env, recurring_id: u32) { execute_recurring(&e, recurring_id) }
    pub fn cancel_recurring(e: Env, caller: Address, recurring_id: u32) { cancel_recurring(&e, caller, recurring_id) }
    pub fn get_recurring(e: Env, recurring_id: u32) -> RecurringRecord { get_recurring(&e, recurring_id) }
        setup_recurring(&e, payer, payee, amount, interval)
    }
    pub fn execute_recurring(e: Env, recurring_id: u32) { execute_recurring(&e, recurring_id) }
    pub fn cancel_recurring(e: Env, caller: Address, recurring_id: u32) { cancel_recurring(&e, caller, recurring_id) }
    pub fn get_recurring(e: Env, recurring_id: u32) -> RecurringRecord { get_recurring(&e, recurring_id) }
        setup_recurring(&e, payer, payee, amount, interval)
    }
    pub fn execute_recurring(e: Env, recurring_id: u32) { execute_recurring(&e, recurring_id) }
    pub fn cancel_recurring(e: Env, caller: Address, recurring_id: u32) { cancel_recurring(&e, caller, recurring_id) }
    pub fn get_recurring(e: Env, recurring_id: u32) -> RecurringRecord { get_recurring(&e, recurring_id) }
    // --- Recurring Payments ---

    pub fn setup_recurring(e: Env, payer: Address, payee: Address, amount: i128, interval: u32) -> u32 {
        setup_recurring(&e, payer, payee, amount, interval)
    }

    pub fn execute_recurring(e: Env, recurring_id: u32) {
        execute_recurring(&e, recurring_id)
    }

    pub fn cancel_recurring(e: Env, caller: Address, recurring_id: u32) {
        cancel_recurring(&e, caller, recurring_id)
    }

    pub fn get_recurring(e: Env, recurring_id: u32) -> RecurringRecord {
        get_recurring(&e, recurring_id)
    }

    pub fn recurring_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::RecurringCount)
    }
    pub fn amend_recurring(e: Env, caller: Address, recurring_id: u32, new_amount: Option<i128>, new_interval: Option<u32>) {
        amend_recurring(&e, caller, recurring_id, new_amount, new_interval)
    pub fn get_recurring_by_payer(e: Env, payer: Address) -> Vec<u32> {
        get_recurring_by_payer(&e, payer)

    pub fn pause_recurring(e: Env, caller: Address, recurring_id: u32) {
        pause_recurring(&e, caller, recurring_id)
    }

    pub fn resume_recurring(e: Env, caller: Address, recurring_id: u32) {
        resume_recurring(&e, caller, recurring_id)
    }
}