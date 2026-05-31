use soroban_sdk::{Env, Symbol, Bytes, Address};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub ledger: u32,
    pub reserve_a: i128,
    pub reserve_b: i128,
    pub total_shares: i128,
    pub accrued_fee_a: i128,
    pub accrued_fee_b: i128,
    pub price_range_low: i128,
    pub price_range_high: i128,
}

pub const MAX_SNAPSHOTS: usize = 1000; // simple cap

pub fn snapshot_position(env: &Env) {
    let ledger = env.ledger().sequence();
    let reserve_a = super::AmmPool::get_reserve_a(env.clone());
    let reserve_b = super::AmmPool::get_reserve_b(env.clone());
    let total_shares = super::AmmPool::get_total_shares(env.clone());
    let accrued_a = env.storage().instance().get(&super::DataKey::AccruedFeeA).unwrap_or(0);
    let accrued_b = env.storage().instance().get(&super::DataKey::AccruedFeeB).unwrap_or(0);
    // simple price range: +/-10% around current price
    let price = reserve_b * 1_000_000 / reserve_a; // scaled price
    let low = price * 9 / 10;
    let high = price * 11 / 10;
    let snap = Snapshot {
        ledger,
        reserve_a,
        reserve_b,
        total_shares,
        accrued_fee_a: accrued_a,
        accrued_fee_b: accrued_b,
        price_range_low: low,
        price_range_high: high,
    };
    // store in a vector under a storage key
    let mut snaps: Vec<Snapshot> = env.storage().instance().get(&super::DataKey::Snapshots).unwrap_or_else(|| Vec::new());
    if snaps.len() >= MAX_SNAPSHOTS {
        snaps.remove(0);
    }
    snaps.push(snap);
    env.storage().instance().set(&super::DataKey::Snapshots, &snaps);
}
