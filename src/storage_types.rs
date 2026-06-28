/// The maximum amount of tokens a single standard escrow can lock.
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
    Frozen(Address),
    EscrowDispute(u32),
    LastEscrowTime(Address),
    TotalSupply,
    RecurringCount,
    Recurring(u32),
}
