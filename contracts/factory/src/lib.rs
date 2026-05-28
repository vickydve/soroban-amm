//! Pool registry. Maps `(token_a, token_b)` pairs to deployed AMM pool addresses
//! and tracks the set of tokens each token has been paired with so a router can
//! discover multi-hop paths without hardcoding addresses.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contracttype]
pub enum DataKey {
    Admin,
    Pool(Address, Address),
    Partners(Address),
}

#[contract]
pub struct Factory;

#[contractimpl]
impl Factory {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Register a deployed AMM pool for the `(token_a, token_b)` pair.
    /// Admin only. Stores both orderings so callers can look up either way.
    pub fn register_pool(env: Env, pool: Address, token_a: Address, token_b: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        assert!(token_a != token_b, "tokens must differ");

        let key_ab = DataKey::Pool(token_a.clone(), token_b.clone());
        if env.storage().persistent().has(&key_ab) {
            panic!("pool already registered");
        }
        let key_ba = DataKey::Pool(token_b.clone(), token_a.clone());

        env.storage().persistent().set(&key_ab, &pool);
        env.storage().persistent().set(&key_ba, &pool);

        Self::add_partner(&env, &token_a, &token_b);
        Self::add_partner(&env, &token_b, &token_a);

        env.events().publish(
            (Symbol::new(&env, "register_pool"), pool),
            (token_a, token_b),
        );
    }

    pub fn get_pool(env: Env, token_a: Address, token_b: Address) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Pool(token_a, token_b))
    }

    pub fn get_partners(env: Env, token: Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Partners(token))
            .unwrap_or_else(|| Vec::new(&env))
    }

    fn add_partner(env: &Env, token: &Address, partner: &Address) {
        let key = DataKey::Partners(token.clone());
        let mut partners: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        for p in partners.iter() {
            if &p == partner {
                return;
            }
        }
        partners.push_back(partner.clone());
        env.storage().persistent().set(&key, &partners);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_register_and_lookup() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let factory_addr = env.register_contract(None, Factory);
        let factory = FactoryClient::new(&env, &factory_addr);
        factory.initialize(&admin);

        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let pool = Address::generate(&env);

        factory.register_pool(&pool, &token_a, &token_b);

        assert_eq!(factory.get_pool(&token_a, &token_b), Some(pool.clone()));
        assert_eq!(factory.get_pool(&token_b, &token_a), Some(pool));

        let partners_a = factory.get_partners(&token_a);
        assert_eq!(partners_a.len(), 1);
        assert_eq!(partners_a.get(0).unwrap(), token_b);
    }

    #[test]
    #[should_panic(expected = "pool already registered")]
    fn test_double_register_panics() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let factory_addr = env.register_contract(None, Factory);
        let factory = FactoryClient::new(&env, &factory_addr);
        factory.initialize(&admin);

        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let pool = Address::generate(&env);

        factory.register_pool(&pool, &token_a, &token_b);
        factory.register_pool(&pool, &token_a, &token_b);
    }
}
