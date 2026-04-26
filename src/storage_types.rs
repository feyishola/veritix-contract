use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    EscrowCount,
    Escrow(u32),
    DepositorEscrows(Address),
    BeneficiaryEscrows(Address), // NEW — #177
    // Reserved for multi-party escrow — see #176
    MultiEscrowCount,
    MultiEscrow(u32),
}
