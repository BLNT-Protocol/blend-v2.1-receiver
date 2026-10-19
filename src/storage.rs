use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Address, Env};

const ONE_DAY_LEDGERS: u32 = 17_280;
const TTL_THRESHOLD: u32 = 90 * ONE_DAY_LEDGERS;
const TTL_BUMP: u32 = 180 * ONE_DAY_LEDGERS;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Config {
    pub blnd_token: Address,
    pub emitter: Address,
    pub initial_backstop: Address,
    pub legacy_backstop_token: Address,
    pub v2_1_backstop: Address,
    pub v2_1_backstop_token: Address,
    pub ysc_recipient: Address,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Config,
    Lock,
}

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_BUMP);
}

pub fn set_config(e: &Env, config: &Config) {
    e.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(e: &Env) -> Config {
    e.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_optimized()
}

pub fn get_lock(e: &Env) -> bool {
    e.storage().instance().get(&DataKey::Lock).unwrap_or(false)
}

pub fn set_lock(e: &Env, value: bool) {
    e.storage().instance().set(&DataKey::Lock, &value);
}
