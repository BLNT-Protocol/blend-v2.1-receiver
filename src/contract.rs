use crate::{
    emitter::{EmitterClient, Swap},
    errors::ReceiverError,
    storage::{self, Config},
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token::TokenClient, Address, Env, Executable,
};

#[contract]
pub struct BlendV21Receiver;

fn enter(e: &Env) {
    if storage::get_lock(e) {
        panic_with_error!(e, ReceiverError::ReentrantCall);
    }
    storage::set_lock(e, true);
}

fn exit(e: &Env) {
    storage::set_lock(e, false);
}

fn require_backstop(e: &Env, emitter: &EmitterClient, expected: &Address) {
    if emitter.get_backstop() != *expected {
        panic_with_error!(e, ReceiverError::UnexpectedBackstop);
    }
}

fn require_swap(e: &Env, emitter: &EmitterClient, backstop: &Address, token: &Address) -> Swap {
    let swap = emitter
        .get_queued_swap()
        .unwrap_or_else(|| panic_with_error!(e, ReceiverError::UnexpectedSwap));
    if swap.new_backstop != *backstop || swap.new_backstop_token != *token {
        panic_with_error!(e, ReceiverError::UnexpectedSwap);
    }
    swap
}

fn sweep(e: &Env, token: &Address, recipient: &Address) -> i128 {
    let contract = e.current_contract_address();
    let client = TokenClient::new(e, token);
    let amount = client.balance(&contract);
    if amount == 0 {
        return 0;
    }
    let recipient_before = client.balance(recipient);
    client.transfer(&contract, recipient, &amount);
    if client.balance(&contract) != 0
        || client.balance(recipient).checked_sub(recipient_before) != Some(amount)
    {
        panic_with_error!(e, ReceiverError::BalanceMismatch);
    }
    amount
}

fn burn_blnd(e: &Env, token: &Address) -> i128 {
    let contract = e.current_contract_address();
    let client = TokenClient::new(e, token);
    let amount = client.balance(&contract);
    if amount == 0 {
        return 0;
    }
    client.burn(&contract, &amount);
    if client.balance(&contract) != 0 {
        panic_with_error!(e, ReceiverError::BalanceMismatch);
    }
    amount
}

#[contractimpl]
impl BlendV21Receiver {
    pub fn __constructor(
        e: Env,
        emitter: Address,
        legacy_backstop_token: Address,
        v2_1_backstop: Address,
        v2_1_backstop_token: Address,
        blnd_token: Address,
        ysc_recipient: Address,
    ) {
        let contract = e.current_contract_address();
        if emitter.executable().is_none()
            || legacy_backstop_token.executable().is_none()
            || v2_1_backstop.executable().is_none()
            || v2_1_backstop_token.executable().is_none()
            || blnd_token.executable() != Some(Executable::StellarAsset)
            || legacy_backstop_token == v2_1_backstop_token
            || legacy_backstop_token == blnd_token
            || v2_1_backstop_token == blnd_token
            || v2_1_backstop == contract
            || ysc_recipient == contract
            || TokenClient::new(&e, &legacy_backstop_token).decimals() != 7
            || TokenClient::new(&e, &v2_1_backstop_token).decimals() != 7
            || TokenClient::new(&e, &blnd_token).decimals() != 7
        {
            panic_with_error!(&e, ReceiverError::InvalidConfiguration);
        }

        let initial_backstop = EmitterClient::new(&e, &emitter).get_backstop();
        if initial_backstop == contract || initial_backstop == v2_1_backstop {
            panic_with_error!(&e, ReceiverError::InvalidConfiguration);
        }

        storage::set_config(
            &e,
            &Config {
                blnd_token,
                emitter,
                initial_backstop,
                legacy_backstop_token,
                v2_1_backstop,
                v2_1_backstop_token,
                ysc_recipient,
            },
        );
        storage::extend_instance(&e);
    }

    /// Queue this contract as the emitter's intermediate backstop while fixing
    /// the token selected for the next comparison to the V2.1 backstop token.
    pub fn queue_receiver(e: Env) {
        storage::extend_instance(&e);
        enter(&e);
        let config = storage::get_config(&e);
        let emitter = EmitterClient::new(&e, &config.emitter);
        require_backstop(&e, &emitter, &config.initial_backstop);
        if emitter.get_queued_swap().is_some() {
            panic_with_error!(&e, ReceiverError::UnexpectedSwap);
        }

        let contract = e.current_contract_address();
        emitter.queue_swap_backstop(&contract, &config.v2_1_backstop_token);
        exit(&e);
    }

    /// Finalize the receiver swap, immediately queue the fixed V2.1 backstop,
    /// and return all legacy LP to the fixed YSC recipient atomically.
    pub fn advance_to_v2_1(e: Env) -> i128 {
        storage::extend_instance(&e);
        enter(&e);
        let config = storage::get_config(&e);
        let emitter = EmitterClient::new(&e, &config.emitter);
        let contract = e.current_contract_address();
        let current = emitter.get_backstop();

        if current == config.initial_backstop {
            require_swap(&e, &emitter, &contract, &config.v2_1_backstop_token);
            emitter.swap_backstop();
            require_backstop(&e, &emitter, &contract);
        } else if current != contract {
            panic_with_error!(&e, ReceiverError::UnexpectedBackstop);
        }

        let swept_new = sweep(&e, &config.v2_1_backstop_token, &config.ysc_recipient);
        match emitter.get_queued_swap() {
            Some(swap)
                if swap.new_backstop == config.v2_1_backstop
                    && swap.new_backstop_token == config.v2_1_backstop_token => {}
            Some(_) => panic_with_error!(&e, ReceiverError::UnexpectedSwap),
            None => emitter.queue_swap_backstop(&config.v2_1_backstop, &config.v2_1_backstop_token),
        }
        let returned = sweep(&e, &config.legacy_backstop_token, &config.ysc_recipient);
        exit(&e);
        let _ = swept_new;
        returned
    }

    /// Finalize the queued V2.1 swap and atomically burn all normal BLND that
    /// the unchanged emitter minted to the receiver during its active period.
    pub fn finalize_v2_1(e: Env) -> i128 {
        storage::extend_instance(&e);
        enter(&e);
        let config = storage::get_config(&e);
        let emitter = EmitterClient::new(&e, &config.emitter);
        let contract = e.current_contract_address();
        let current = emitter.get_backstop();

        if current == contract {
            require_swap(
                &e,
                &emitter,
                &config.v2_1_backstop,
                &config.v2_1_backstop_token,
            );
            sweep(&e, &config.v2_1_backstop_token, &config.ysc_recipient);
            emitter.swap_backstop();
            require_backstop(&e, &emitter, &config.v2_1_backstop);
        } else if current != config.v2_1_backstop {
            panic_with_error!(&e, ReceiverError::UnexpectedBackstop);
        }

        let burned = burn_blnd(&e, &config.blnd_token);
        sweep(&e, &config.legacy_backstop_token, &config.ysc_recipient);
        sweep(&e, &config.v2_1_backstop_token, &config.ysc_recipient);
        exit(&e);
        burned
    }

    /// Return legacy LP after the receiver has become the current backstop.
    /// This is a permissionless recovery path if the emitter swap was called
    /// directly instead of through `advance_to_v2_1`.
    pub fn return_legacy(e: Env) -> i128 {
        storage::extend_instance(&e);
        enter(&e);
        let config = storage::get_config(&e);
        let current = EmitterClient::new(&e, &config.emitter).get_backstop();
        let contract = e.current_contract_address();
        if current != contract && current != config.v2_1_backstop {
            panic_with_error!(&e, ReceiverError::UnexpectedBackstop);
        }
        let returned = sweep(&e, &config.legacy_backstop_token, &config.ysc_recipient);
        exit(&e);
        returned
    }

    /// Burn any BLND sent or normally emitted to this receiver.
    pub fn burn_received_blnd(e: Env) -> i128 {
        storage::extend_instance(&e);
        enter(&e);
        let burned = burn_blnd(&e, &storage::get_config(&e).blnd_token);
        exit(&e);
        burned
    }

    pub fn get_config(e: Env) -> Config {
        storage::get_config(&e)
    }
}
