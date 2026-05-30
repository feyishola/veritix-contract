use crate::admin::{check_admin, has_admin, read_admin, transfer_admin, write_admin};
use crate::allowance::{get_allowances_for_spender, read_allowance, spend_allowance, write_allowance};
use crate::batch::{clawback_batch, freeze_batch, unfreeze_batch};
use crate::balance::{
    decrease_supply, increase_supply, read_balance, read_total_supply, receive_balance,
    spend_balance,
};
use crate::dispute::{get_dispute as dispute_get, open_dispute, resolve_dispute, DisputeRecord};
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
    cancel_recurring, execute_recurring, get_recurring, setup_recurring, RecurringRecord,
};
use crate::splitter::{
    cancel_split as split_cancel, create_split as split_create, distribute as split_distribute,
    get_split as split_get, SplitRecord, SplitRecipient,
};
use crate::validation::{require_not_frozen_account, require_positive_amount};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec};

#[contract]
pub struct VeritixToken;

#[derive(Clone)]
#[contracttype]
pub struct AdminInfo {
    pub admin: Address,
    pub paused: bool,
}

#[contractimpl]
impl VeritixToken {
    // --- Admin & metadata ---

    /// Sets admin and metadata. Panics if already initialized.
    pub fn initialize(e: Env, admin: Address, name: String, symbol: String, decimal: u32) {
        if has_admin(&e) {
            panic!("already initialized");
        }

        admin.require_auth();

        let meta = TokenMetadata {
            name,
            symbol,
            decimal,
        };
        validate_metadata(&meta);
        write_admin(&e, &admin);
        write_metadata(&e, meta);
    }

    /// Rotates the contract administrator. Requires current admin auth.
    pub fn set_admin(e: Env, new_admin: Address) {
        transfer_admin(&e, new_admin);
    }

    /// Admin-only. Reclaims tokens from an address and destroys them.
    pub fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_admin(&e, &admin);
        require_positive_amount(amount);

        // Deduct balance without redistributing, effectively burning the tokens
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);

        // Emit transparency event
        e.events()
            .publish((symbol_short!("clawback"), admin, from), amount);
    }
    /// Admin-only batch clawback over `(from, amount)` tuples.
    pub fn clawback_batch(e: Env, admin: Address, targets: Vec<(Address, i128)>) {
        clawback_batch(&e, admin, targets);
    }

    // --- Freeze controls ---

    /// Admin-only freeze for a single account.
    pub fn freeze(e: Env, target: Address) {
        let admin = read_admin(&e);
        check_admin(&e, &admin);
        freeze_account(&e, admin, target);
    }

    /// Admin-only unfreeze for a single account.
    pub fn unfreeze(e: Env, target: Address) {
        let admin = read_admin(&e);
        check_admin(&e, &admin);
        unfreeze_account(&e, admin, target);
    }
    /// Admin-only batch freeze for multiple accounts.
    pub fn freeze_batch(e: Env, admin: Address, targets: Vec<Address>) {
        freeze_batch(&e, admin, targets);
    }

    /// Admin-only batch unfreeze for multiple accounts.
    pub fn unfreeze_batch(e: Env, admin: Address, targets: Vec<Address>) {
        unfreeze_batch(&e, admin, targets);
    }

    // --- Mint / burn & supply tracking ---

    /// Admin-only. Mints new tokens to a specific address.
    pub fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_admin(&e, &admin);
        require_positive_amount(amount);
        receive_balance(&e, to.clone(), amount);
        increase_supply(&e, amount);
        e.events()
            .publish((symbol_short!("mint"), admin, to), amount);
    }

    /// Caller burns their own tokens.
    pub fn burn(e: Env, from: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_positive_amount(amount);
        from.require_auth();
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);
        e.events().publish((symbol_short!("burn"), from), amount);
    }

    /// Spender burns tokens from an account using their allowance.
    pub fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_not_frozen_account(&e, &spender);
        require_positive_amount(amount);
        spender.require_auth();
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount);
        decrease_supply(&e, amount);
        e.events()
            .publish((symbol_short!("burn"), spender, from), amount);
    }

    // --- Transfers & allowance ---

    /// Standard token transfer between two addresses.
    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_positive_amount(amount);
        from.require_auth();
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        e.events()
            .publish((symbol_short!("transfer"), from, to), amount);
    }

    /// Transfer tokens on behalf of a user via allowance.
    pub fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        require_not_frozen_account(&e, &from);
        require_not_frozen_account(&e, &spender);
        require_positive_amount(amount);
        spender.require_auth();
        spend_allowance(&e, from.clone(), spender.clone(), amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        e.events()
            .publish((symbol_short!("transfer"), from, to), amount);
    }

    /// Sets an allowance for a spender.
    /// Frozen accounts cannot create new approvals.
    pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        require_not_frozen_account(&e, &from);
        from.require_auth();
        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        e.events()
            .publish((symbol_short!("approve"), from, spender), amount);
    }

    // --- Read-only views ---

    /// Returns current total token supply.
    pub fn total_supply(e: Env) -> i128 {
        read_total_supply(&e)
    }

    /// Returns token balance for `id`.
    pub fn balance(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    /// Returns allowance from `from` to `spender`.
    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&e, from, spender).amount
    }
    /// Returns all owners that granted non-zero allowance to `spender`.
    pub fn allowances_for_spender(e: Env, spender: Address) -> Vec<Address> {
        get_allowances_for_spender(&e, spender)
    }

    /// Returns current admin address.
    pub fn admin(e: Env) -> Address {
        read_admin(&e)
    }
    /// Returns compact admin metadata for clients.
    pub fn admin_info(e: Env) -> AdminInfo {
        AdminInfo {
            admin: read_admin(&e),
            paused: false,
        }
    }

    /// Returns freeze state for account `id`.
    pub fn is_frozen(e: Env, id: Address) -> bool {
        read_frozen_status(&e, &id)
    }

    /// Returns token decimal precision.
    pub fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    /// Returns token display name.
    pub fn name(e: Env) -> String {
        read_name(&e)
    }

    /// Returns token symbol.
    pub fn symbol(e: Env) -> String {
        read_symbol(&e)
    }

    // --- Escrow ---

    /// Creates an escrow and returns its ID.
    pub fn create_escrow(e: Env, depositor: Address, beneficiary: Address, amount: i128, expiry_ledger: u32) -> u32 {
        escrow_create(&e, depositor, beneficiary, amount, expiry_ledger)
    }

    /// Releases escrow funds to beneficiary.
    pub fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow_release(&e, caller, escrow_id)
    }

    /// Refunds escrow funds to depositor.
    pub fn refund_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow_refund(&e, caller, escrow_id)
    }

    /// Returns escrow record for `escrow_id`.
    pub fn get_escrow(e: Env, escrow_id: u32) -> EscrowRecord {
        escrow_get(&e, escrow_id)
    }

    /// Returns the current number of escrows created (monotonically increasing counter).
    pub fn escrow_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::EscrowCount)
    }

    /// Admin escape hatch: forcibly settles a stuck escrow when the normal
    /// beneficiary or depositor is frozen. Sends funds to `recipient`.
    pub fn admin_settle_escrow(e: Env, admin: Address, escrow_id: u32, recipient: Address) {
        escrow_admin_settle(&e, admin, escrow_id, recipient)
    }

    // --- Dispute ---

    /// Opens a dispute for an escrow and returns dispute ID.
    pub fn open_dispute(e: Env, claimant: Address, escrow_id: u32, resolver: Address) -> u32 {
        open_dispute(&e, claimant, escrow_id, resolver)
    }

    /// Resolves dispute by releasing to beneficiary or refunding depositor.
    pub fn resolve_dispute(
        e: Env,
        resolver: Address,
        dispute_id: u32,
        release_to_beneficiary: bool,
    ) {
        resolve_dispute(&e, resolver, dispute_id, release_to_beneficiary)
    }

    /// Returns dispute record for `dispute_id`.
    pub fn get_dispute(e: Env, dispute_id: u32) -> DisputeRecord {
        dispute_get(&e, dispute_id)
    }

    /// Returns the current number of disputes created (monotonically increasing counter).
    pub fn dispute_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::DisputeCount)
    }

    // --- Splitter ---

    /// Creates a split payment plan and returns split ID.
    pub fn create_split(
        e: Env,
        sender: Address,
        recipients: Vec<SplitRecipient>,
        total_amount: i128,
    ) -> u32 {
        split_create(&e, sender, recipients, total_amount)
    }

    /// Executes split distribution to recipients.
    pub fn distribute(e: Env, caller: Address, split_id: u32) {
        split_distribute(&e, caller, split_id)
    }

    /// Cancels an active split and returns remainder to sender.
    pub fn cancel_split(e: Env, caller: Address, split_id: u32) {
        split_cancel(&e, caller, split_id)
    }

    /// Returns split record for `split_id`.
    pub fn get_split(e: Env, split_id: u32) -> SplitRecord {
        split_get(&e, split_id)
    }

    /// Returns the current number of splits created (monotonically increasing counter).
    pub fn split_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::SplitCount)
    }

    // --- Recurring Payments ---

    /// Creates a recurring payment and returns recurring ID.
    pub fn setup_recurring(
        e: Env,
        payer: Address,
        payee: Address,
        amount: i128,
        interval: u32,
    ) -> u32 {
        setup_recurring(&e, payer, payee, amount, interval)
    }

    /// Executes one interval payment for a recurring plan.
    pub fn execute_recurring(e: Env, recurring_id: u32) {
        execute_recurring(&e, recurring_id)
    }

    /// Cancels recurring payment plan.
    pub fn cancel_recurring(e: Env, caller: Address, recurring_id: u32) {
        cancel_recurring(&e, caller, recurring_id)
    }

    /// Returns recurring payment record for `recurring_id`.
    pub fn get_recurring(e: Env, recurring_id: u32) -> RecurringRecord {
        get_recurring(&e, recurring_id)
    }

    /// Returns the current number of recurring payments created (monotonically increasing counter).
    pub fn recurring_count(e: Env) -> u32 {
        crate::storage_types::bump_instance(&e);
        crate::storage_types::read_counter(&e, &crate::storage_types::DataKey::RecurringCount)
    }
}
