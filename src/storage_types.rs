// Append or insert into your existing storage_types.rs file

/// The maximum amount of tokens a single standard escrow can lock.
/// 
/// **Rationale:** Restricting a single escrow to 1% of the theoretical supply boundary 
/// (`i128::MAX / 100`) prevents complete token ecosystem paralysis if a massive escrow 
/// becomes permanently bricked (e.g., dead beneficiary key, missed expiration, or lost dispute access).
pub const MAX_ESCROW_AMOUNT: i128 = i128::MAX / 100;
use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    EscrowCount,
    Escrow(u32),
    DepositorEscrows(Address),
    BeneficiaryEscrows(Address),
    MultiEscrowCount,
    MultiEscrow(u32),
    Allowance(Address, Address),
}