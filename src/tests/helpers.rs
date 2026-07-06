use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};

use crate::contract::{StreamFundContract, StreamFundContractClient};

pub const START_OFFSET: u64 = 100;   // seconds after "now" that streams start
pub const DURATION: u64 = 1_000;     // stream duration in seconds
pub const AMOUNT: i128 = 1_000_000;  // total tokens escrowed

/// Bootstrap a fresh Soroban test environment.
///
/// Returns all components needed by a test. The `env` must outlive the
/// `client` — callers must keep both in the same scope.
pub fn setup() -> (Env, Address, Address, Address, Address, u64, u64, u64) {
    let env = Env::default();
    env.mock_all_auths();

    let now: u64 = 1_000_000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let contract_id = env.register(StreamFundContract, ());
    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token_id).mint(&sender, &(AMOUNT * 10));

    let start = now + START_OFFSET;
    let stop = start + DURATION;

    (env, contract_id, token_id, sender, receiver, now, start, stop)
}

/// Advance the ledger timestamp by `delta` seconds.
pub fn advance(env: &Env, delta: u64) {
    let ts = env.ledger().timestamp();
    env.ledger().with_mut(|l| l.timestamp = ts + delta);
}

/// Set the ledger timestamp to an absolute value.
pub fn set_time(env: &Env, ts: u64) {
    env.ledger().with_mut(|l| l.timestamp = ts);
}

/// Create a default stream with stream_id = 1 and return its id.
pub fn create_default_stream(
    client: &StreamFundContractClient,
    token_id: &Address,
    sender: &Address,
    receiver: &Address,
    start: u64,
    stop: u64,
) -> u64 {
    let id = 1u64;
    client
        .create_stream(&id, sender, receiver, token_id, &AMOUNT, &start, &stop)
        .unwrap();
    id
}

/// Return the token balance of an address.
pub fn balance(env: &Env, token_id: &Address, addr: &Address) -> i128 {
    TokenClient::new(env, token_id).balance(addr)
}
