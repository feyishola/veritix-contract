use crate::admin::check_admin;
use crate::balance::read_balance;
use crate::storage_types::{increment_counter, DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone)]
pub struct SnapshotRecord {
    pub id: u32,
    pub ledger: u32,
    pub balances: Vec<(Address, i128)>,
}

pub fn take_snapshot(e: &Env, admin: Address, addresses: Vec<Address>) -> u32 {
    check_admin(e, &admin);
    let ledger = e.ledger().sequence();
    let snapshot_id = increment_counter(e, &DataKey::SnapshotCount);
    let mut balances: Vec<(Address, i128)> = Vec::new(e);
    for i in 0..addresses.len() {
        let addr = addresses.get(i).unwrap();
        let balance = read_balance(e, addr.clone());
        balances.push_back((addr, balance));
    }
    let record = SnapshotRecord {
        id: snapshot_id,
        ledger,
        balances,
    };
    let key = DataKey::Snapshot(snapshot_id);
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    snapshot_id
}

pub fn get_snapshot_balance(e: &Env, snapshot_id: u32, address: Address) -> i128 {
    let key = DataKey::Snapshot(snapshot_id);
    let record: SnapshotRecord = e.storage().persistent().get(&key).expect("snapshot not found");
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    for i in 0..record.balances.len() {
        let (addr, balance) = record.balances.get(i).unwrap();
        if addr == address {
            return balance;
        }
    }
    0
}

pub fn get_snapshot_ledger(e: &Env, snapshot_id: u32) -> u32 {
    let key = DataKey::Snapshot(snapshot_id);
    let record: SnapshotRecord = e.storage().persistent().get(&key).expect("snapshot not found");
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    record.ledger
}
