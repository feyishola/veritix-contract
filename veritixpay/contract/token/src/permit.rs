use soroban_sdk::{symbol_short, xdr::ToXdr, Bytes, BytesN, Env, Address};
use crate::allowance::write_allowance;
use crate::storage_types::{DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};

fn read_nonce(e: &Env, owner: &Address) -> u64 {
    let key = DataKey::Nonce(owner.clone());
    e.storage().persistent().get(&key).unwrap_or(0)
}

fn write_nonce(e: &Env, owner: &Address, nonce: u64) {
    let key = DataKey::Nonce(owner.clone());
    e.storage().persistent().set(&key, &nonce);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn permit(
    e: &Env,
    owner: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
    nonce: u64,
    public_key: BytesN<32>,
    signature: BytesN<64>,
) {
    let current_nonce = read_nonce(e, &owner);
    if nonce != current_nonce {
        panic!("invalid nonce");
    }

    let hash = hash_permit(e, &owner, &spender, amount, expiration_ledger, nonce);
    let hash_bytes: Bytes = hash.into();
    e.crypto().ed25519_verify(&public_key, &hash_bytes, &signature);

    write_nonce(e, &owner, current_nonce + 1);
    write_allowance(e, owner, spender, amount, expiration_ledger);
}

pub fn nonces(e: &Env, owner: Address) -> u64 {
    read_nonce(e, &owner)
}

fn hash_permit(
    e: &Env,
    owner: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
    nonce: u64,
) -> BytesN<32> {
    let mut msg = soroban_sdk::Bytes::new(e);
    msg.append(&symbol_short!("permit").to_xdr(e));
    msg.append(&owner.clone().to_xdr(e));
    msg.append(&spender.clone().to_xdr(e));
    msg.append(&amount.to_xdr(e));
    msg.append(&expiration_ledger.to_xdr(e));
    msg.append(&nonce.to_xdr(e));
    e.crypto().sha256(&msg)
}
