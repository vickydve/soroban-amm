#![no_std]

use soroban_sdk::{contract, contractclient, contractimpl, contracttype, Address, Env, Symbol};

#[contractclient(name = "LiquidityPoolOracleClient")]
pub trait LiquidityPoolOracle {
    fn get_liquidity_cumulative(env: Env) -> (i128, u64);
}

#[contracttype]
pub enum DataKey {
    Keeper,
    LiquiditySnapshot(Address, u64),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquiditySnapshot {
    pub cumulative_liquidity: i128,
    pub pool_ts: u64,
}

#[contract]
pub struct TwalConsumer;

#[contractimpl]
impl TwalConsumer {
    /// Keep snapshot alive for 7 days (in ledgers: 7 * 24 * 3600 / 5 ≈ 120,960).
    pub const SNAPSHOT_TTL_LEDGERS: u32 = 120_960;

    pub fn initialize(env: Env, keeper: Address) {
        assert!(
            !env.storage().instance().has(&DataKey::Keeper),
            "already initialized"
        );
        env.storage().instance().set(&DataKey::Keeper, &keeper);
    }

    pub fn keeper(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Keeper).unwrap()
    }

    /// Stores a pool cumulative-liquidity snapshot keyed by the keeper clock.
    pub fn save_snapshot(env: Env, pool: Address) {
        Self::require_keeper(&env);
        let (cumulative_liquidity, pool_ts) =
            LiquidityPoolOracleClient::new(&env, &pool).get_liquidity_cumulative();
        let ledger_ts = env.ledger().timestamp();
        let snapshot = LiquiditySnapshot {
            cumulative_liquidity,
            pool_ts,
        };
        let key = DataKey::LiquiditySnapshot(pool, ledger_ts);
        env.storage().persistent().set(&key, &snapshot);
        env.storage().persistent().extend_ttl(
            &key,
            Self::SNAPSHOT_TTL_LEDGERS / 2,
            Self::SNAPSHOT_TTL_LEDGERS,
        );
    }

    /// Deletes a liquidity snapshot from persistent storage. Keeper-only.
    pub fn delete_snapshot(env: Env, pool: Address, ledger_ts: u64) {
        Self::require_keeper(&env);
        let key = DataKey::LiquiditySnapshot(pool.clone(), ledger_ts);
        env.storage().persistent().remove(&key);
        env.events()
            .publish((Symbol::new(&env, "snapshot_deleted"),), (pool, ledger_ts));
    }

    /// Computes TWAL over `window_seconds` from a stored cumulative-liquidity snapshot.
    pub fn get_twal_liquidity(env: Env, pool: Address, window_seconds: u64) -> i128 {
        assert!(window_seconds > 0, "window_seconds must be > 0");
        let (cumulative_now, pool_ts_now) =
            LiquidityPoolOracleClient::new(&env, &pool).get_liquidity_cumulative();
        let ledger_ts_now = env.ledger().timestamp();
        assert!(
            ledger_ts_now >= window_seconds,
            "ledger timestamp is smaller than requested window"
        );

        let then_ts = ledger_ts_now - window_seconds;
        let key = DataKey::LiquiditySnapshot(pool, then_ts);
        let snapshot: LiquiditySnapshot =
            env.storage().persistent().get(&key).unwrap_or_else(|| {
                panic!("missing liquidity snapshot at target ledger timestamp {then_ts}")
            });
        env.storage().persistent().extend_ttl(
            &key,
            Self::SNAPSHOT_TTL_LEDGERS / 2,
            Self::SNAPSHOT_TTL_LEDGERS,
        );

        let elapsed = (pool_ts_now - snapshot.pool_ts) as i128;
        assert!(elapsed > 0, "window too small (pool time did not advance)");
        (cumulative_now - snapshot.cumulative_liquidity) / elapsed
    }

    fn require_keeper(env: &Env) {
        let keeper: Address = env.storage().instance().get(&DataKey::Keeper).unwrap();
        keeper.require_auth();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env,
    };

    #[contract]
    pub struct MockLiquidityPool;

    #[contracttype]
    enum MockKey {
        Cumulative,
        Timestamp,
    }

    #[contractimpl]
    impl MockLiquidityPool {
        pub fn set(env: Env, cumulative: i128, timestamp: u64) {
            env.storage()
                .instance()
                .set(&MockKey::Cumulative, &cumulative);
            env.storage()
                .instance()
                .set(&MockKey::Timestamp, &timestamp);
        }

        pub fn get_liquidity_cumulative(env: Env) -> (i128, u64) {
            (
                env.storage()
                    .instance()
                    .get(&MockKey::Cumulative)
                    .unwrap_or(0),
                env.storage()
                    .instance()
                    .get(&MockKey::Timestamp)
                    .unwrap_or(0),
            )
        }
    }

    #[test]
    fn test_delete_snapshot_removes_liquidity_snapshot() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(10_000);
        let keeper = Address::generate(&env);
        let pool_addr = env.register_contract(None, MockLiquidityPool);
        let consumer_addr = env.register_contract(None, TwalConsumer);
        let pool = MockLiquidityPoolClient::new(&env, &pool_addr);
        let consumer = TwalConsumerClient::new(&env, &consumer_addr);

        consumer.initialize(&keeper);
        pool.set(&1_000_i128, &10_000_u64);
        consumer.save_snapshot(&pool_addr);

        env.ledger().set_timestamp(10_100);
        pool.set(&6_000_i128, &10_100_u64);
        assert_eq!(consumer.get_twal_liquidity(&pool_addr, &100_u64), 50);

        consumer.delete_snapshot(&pool_addr, &10_000_u64);
        assert!(consumer
            .try_get_twal_liquidity(&pool_addr, &100_u64)
            .is_err());
    }
}
