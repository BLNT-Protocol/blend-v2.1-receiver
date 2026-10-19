use crate::{BlendV21Receiver, BlendV21ReceiverClient, Swap};
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};

const WAIT_SECONDS: u64 = 31 * 24 * 60 * 60;
const SCALAR_7: i128 = 10_000_000;

#[contract]
struct MockBackstop;

#[contractimpl]
impl MockBackstop {}

#[contract]
struct MockEmitter;

#[contractimpl]
impl MockEmitter {
    pub fn initialize(e: Env, blnd: Address, backstop: Address, token: Address) {
        e.storage().instance().set(&symbol_short!("blnd"), &blnd);
        e.storage()
            .instance()
            .set(&symbol_short!("backstop"), &backstop);
        e.storage().instance().set(&symbol_short!("token"), &token);
        e.storage()
            .instance()
            .set(&symbol_short!("last"), &e.ledger().timestamp());
    }

    pub fn get_backstop(e: Env) -> Address {
        e.storage()
            .instance()
            .get(&symbol_short!("backstop"))
            .unwrap()
    }

    pub fn get_queued_swap(e: Env) -> Option<Swap> {
        e.storage().instance().get(&symbol_short!("queue"))
    }

    pub fn queue_swap_backstop(e: Env, new_backstop: Address, new_backstop_token: Address) {
        assert!(Self::get_queued_swap(e.clone()).is_none());
        let current = Self::get_backstop(e.clone());
        let token: Address = e.storage().instance().get(&symbol_short!("token")).unwrap();
        let client = TokenClient::new(&e, &token);
        assert!(client.balance(&new_backstop) > client.balance(&current));
        let swap = Swap {
            new_backstop,
            new_backstop_token,
            unlock_time: e.ledger().timestamp() + WAIT_SECONDS,
        };
        e.storage().instance().set(&symbol_short!("queue"), &swap);
    }

    pub fn swap_backstop(e: Env) {
        let swap = Self::get_queued_swap(e.clone()).unwrap();
        assert!(e.ledger().timestamp() >= swap.unlock_time);
        let current = Self::get_backstop(e.clone());
        let token: Address = e.storage().instance().get(&symbol_short!("token")).unwrap();
        let client = TokenClient::new(&e, &token);
        assert!(client.balance(&swap.new_backstop) > client.balance(&current));

        let last: u64 = e.storage().instance().get(&symbol_short!("last")).unwrap();
        let blnd: Address = e.storage().instance().get(&symbol_short!("blnd")).unwrap();
        let emissions = i128::from(e.ledger().timestamp() - last) * SCALAR_7;
        StellarAssetClient::new(&e, &blnd).mint(&current, &emissions);

        e.storage()
            .instance()
            .set(&symbol_short!("backstop"), &swap.new_backstop);
        e.storage()
            .instance()
            .set(&symbol_short!("token"), &swap.new_backstop_token);
        e.storage()
            .instance()
            .set(&symbol_short!("last"), &e.ledger().timestamp());
        e.storage().instance().remove(&symbol_short!("queue"));
    }
}

fn create_token(e: &Env, admin: &Address) -> Address {
    e.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

#[test]
fn executes_two_hop_migration_and_returns_legacy_lp() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(1_000_000);

    let emitter = e.register(MockEmitter, ());
    let initial_backstop = Address::generate(&e);
    let v2_1_backstop = e.register(MockBackstop, ());
    let ysc = Address::generate(&e);
    let token_admin = Address::generate(&e);
    let legacy = create_token(&e, &token_admin);
    let v2_1_token = create_token(&e, &token_admin);
    let blnd = create_token(&e, &emitter);
    MockEmitterClient::new(&e, &emitter).initialize(&blnd, &initial_backstop, &legacy);

    let receiver = e.register(
        BlendV21Receiver,
        (&emitter, &legacy, &v2_1_backstop, &v2_1_token, &blnd, &ysc),
    );
    let receiver_client = BlendV21ReceiverClient::new(&e, &receiver);
    let legacy_admin = StellarAssetClient::new(&e, &legacy);
    let v2_1_admin = StellarAssetClient::new(&e, &v2_1_token);
    legacy_admin.mint(&initial_backstop, &(100 * SCALAR_7));
    legacy_admin.mint(&receiver, &(101 * SCALAR_7));
    v2_1_admin.mint(&v2_1_backstop, &(200 * SCALAR_7));
    v2_1_admin.mint(&receiver, &(3 * SCALAR_7));

    receiver_client.queue_receiver();
    let first = MockEmitterClient::new(&e, &emitter)
        .get_queued_swap()
        .unwrap();
    assert_eq!(first.new_backstop, receiver);
    assert_eq!(first.new_backstop_token, v2_1_token);

    e.ledger().set_timestamp(first.unlock_time);
    assert_eq!(receiver_client.advance_to_v2_1(), 101 * SCALAR_7);
    assert_eq!(TokenClient::new(&e, &legacy).balance(&ysc), 101 * SCALAR_7);
    assert_eq!(TokenClient::new(&e, &legacy).balance(&receiver), 0);
    assert_eq!(
        TokenClient::new(&e, &v2_1_token).balance(&ysc),
        3 * SCALAR_7
    );
    assert_eq!(
        MockEmitterClient::new(&e, &emitter).get_backstop(),
        receiver
    );

    let second = MockEmitterClient::new(&e, &emitter)
        .get_queued_swap()
        .unwrap();
    assert_eq!(second.new_backstop, v2_1_backstop);
    assert_eq!(second.new_backstop_token, v2_1_token);

    e.ledger().set_timestamp(second.unlock_time);
    assert_eq!(
        receiver_client.finalize_v2_1(),
        i128::from(WAIT_SECONDS) * SCALAR_7
    );
    assert_eq!(
        MockEmitterClient::new(&e, &emitter).get_backstop(),
        v2_1_backstop
    );
    assert_eq!(TokenClient::new(&e, &blnd).balance(&receiver), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1302)")]
fn rejects_advancing_an_unexpected_queue() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(1_000_000);

    let emitter = e.register(MockEmitter, ());
    let initial_backstop = Address::generate(&e);
    let v2_1_backstop = e.register(MockBackstop, ());
    let wrong_backstop = e.register(MockBackstop, ());
    let ysc = Address::generate(&e);
    let token_admin = Address::generate(&e);
    let legacy = create_token(&e, &token_admin);
    let v2_1_token = create_token(&e, &token_admin);
    let blnd = create_token(&e, &emitter);
    MockEmitterClient::new(&e, &emitter).initialize(&blnd, &initial_backstop, &legacy);
    let receiver = e.register(
        BlendV21Receiver,
        (&emitter, &legacy, &v2_1_backstop, &v2_1_token, &blnd, &ysc),
    );
    StellarAssetClient::new(&e, &legacy).mint(&initial_backstop, &(100 * SCALAR_7));
    StellarAssetClient::new(&e, &legacy).mint(&wrong_backstop, &(101 * SCALAR_7));
    MockEmitterClient::new(&e, &emitter).queue_swap_backstop(&wrong_backstop, &v2_1_token);

    BlendV21ReceiverClient::new(&e, &receiver).advance_to_v2_1();
}
