use crate::admin::check_admin;
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::freeze::is_frozen as read_frozen_status;
use crate::storage_types::DataKey;
use soroban_sdk::{Address, Env, Vec};

pub fn distribute_dividend(e: &Env, admin: Address, amount: i128) {
    check_admin(e, &admin);
    let total_supply = crate::balance::read_total_supply(e);
    if total_supply == 0 {
        panic!("no holders to distribute to");
    }

    let admin_balance = read_balance(e, admin.clone());
    if admin_balance < amount {
        panic!("insufficient dividend pool");
    }
    spend_balance(e, admin.clone(), amount);

    let key = DataKey::HolderSet;
    let holders: Vec<Address> = e.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(e));
    let mut distributed: i128 = 0;

    for i in 0..holders.len() {
        let holder = holders.get(i).unwrap();
        let balance = read_balance(e, holder.clone());
        if balance > 0 && !read_frozen_status(e, &holder) {
            let share = balance * amount / total_supply;
            if share > 0 {
                receive_balance(e, holder.clone(), share);
                distributed += share;
            }
        }
    }

    let remainder = amount - distributed;
    if remainder > 0 {
        receive_balance(e, admin, remainder);
    }
}
