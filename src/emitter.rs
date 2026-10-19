use soroban_sdk::{contractclient, contracttype, Address, Env};

/// The queued-swap value exposed by the unchanged Blend V1 emitter.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Swap {
    pub new_backstop: Address,
    pub new_backstop_token: Address,
    pub unlock_time: u64,
}

#[contractclient(name = "EmitterClient")]
#[allow(dead_code)]
pub trait Emitter {
    fn get_backstop(e: Env) -> Address;
    fn get_queued_swap(e: Env) -> Option<Swap>;
    fn queue_swap_backstop(e: Env, new_backstop: Address, new_backstop_token: Address);
    fn swap_backstop(e: Env);
}
