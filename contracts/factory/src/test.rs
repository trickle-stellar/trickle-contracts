#![cfg(test)]

extern crate std;

use super::*;
use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, testutils::Ledger, Address,
    BytesN, Env, IntoVal, Symbol, Val, Vec,
};
use trickle_common::{StreamError, StreamStatus};

// Imports the stream contract WASM so the factory tests can upload and deploy
// real instances, and provides a client (`stream_wasm::StreamContractClient`)
// to inspect the deployed stream. The types it re-exports are scoped here to
// avoid colliding with the equivalent `trickle_common`/`trickle-stream` types.
mod stream_wasm {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/trickle_stream.wasm");
}

// ─── Mock Token ─────────────────────────────────────────────────────────────
// A minimal token contract exposing the same `balance` / `transfer` symbols as
// the Stellar Asset Contract so the real `soroban_sdk::token::Client` used in
// production code works against it. `transfer` enforces `from` authorization
// exactly like the SAC does.

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
enum TokenDataKey {
    Admin,
    Balance(Address),
}

fn get_balance(env: &Env, address: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&TokenDataKey::Balance(address.clone()))
        .unwrap_or(0)
}

fn set_balance(env: &Env, address: &Address, amount: i128) {
    env.storage()
        .instance()
        .set(&TokenDataKey::Balance(address.clone()), &amount);
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&TokenDataKey::Admin, &admin);
    }

    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&TokenDataKey::Admin)
            .expect("token not initialized");
        if admin != stored_admin {
            panic!("unauthorized mint");
        }
        let balance = get_balance(&env, &to);
        set_balance(&env, &to, balance + amount);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            panic!("insufficient balance");
        }
        let to_balance = get_balance(&env, &to);
        set_balance(&env, &from, from_balance - amount);
        set_balance(&env, &to, to_balance + amount);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        get_balance(&env, &id)
    }
}

// ─── Test Setup ─────────────────────────────────────────────────────────────

struct TestContext {
    env: Env,
    factory: FactoryContractClient<'static>,
    token: MockTokenClient<'static>,
    token_address: Address,
    sender: Address,
    recipient: Address,
}

// amount 10_000 over 100s → flow_rate 100/sec, fully streams out in 100s.
const AMOUNT: i128 = 10_000;
const DURATION: u32 = 100;
const FLOW_RATE: i128 = 100;

fn setup() -> TestContext {
    setup_with_auth(true)
}

fn setup_with_auth(mock_auth: bool) -> TestContext {
    let env = Env::default();
    if mock_auth {
        env.mock_all_auths();
    }

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_address = env.register(MockToken, ());
    let token = MockTokenClient::new(&env, &token_address);
    token.initialize(&admin);
    // Fund the sender generously to cover stream creation plus escrow.
    if mock_auth {
        token.mint(&admin, &sender, &(AMOUNT * 10));
    }

    let wasm_hash = env.deployer().upload_contract_wasm(stream_wasm::WASM);

    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(&env, &factory_id);
    factory.initialize(&admin, &wasm_hash);

    TestContext {
        env,
        factory,
        token,
        token_address,
        sender,
        recipient,
    }
}

fn create_stream(ctx: &TestContext) -> Address {
    ctx.factory.create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &AMOUNT,
        &DURATION,
    )
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let ctx = setup_with_auth(false);
    // Before initialization nothing is registered.
    let empty = ctx.factory.get_streams_by_sender(&ctx.sender);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_get_streams_empty() {
    let ctx = setup_with_auth(false);
    assert_eq!(ctx.factory.get_streams_by_sender(&ctx.sender).len(), 0);
    assert_eq!(
        ctx.factory.get_streams_by_recipient(&ctx.recipient).len(),
        0
    );
}

#[test]
fn test_get_stream_not_found() {
    let ctx = setup();
    assert_eq!(
        ctx.factory.try_get_stream(&0u32),
        Err(Ok(StreamError::StreamNotFound))
    );
}

#[test]
fn test_create_stream_deploys_and_funds_escrow() {
    let ctx = setup();
    let start_time = ctx.env.ledger().timestamp();
    let sender_balance_before = ctx.token.balance(&ctx.sender);

    let stream_address = create_stream(&ctx);

    // Sender debited, and the full amount now sits in the stream's escrow.
    assert_eq!(
        ctx.token.balance(&ctx.sender),
        sender_balance_before - AMOUNT
    );
    assert_eq!(ctx.token.balance(&stream_address), AMOUNT);

    // Registry metadata is cached correctly.
    let info = ctx.factory.get_stream(&0);
    assert_eq!(info.sender, ctx.sender);
    assert_eq!(info.recipient, ctx.recipient);
    assert_eq!(info.asset, ctx.token_address);
    assert_eq!(info.flow_rate, FLOW_RATE);
    assert_eq!(info.total_amount, AMOUNT);
    assert_eq!(info.withdrawn_amount, 0);
    assert_eq!(info.start_time, start_time);
    assert_eq!(info.status, StreamStatus::Active);

    // Indexes: the new stream is listed for both sides.
    let by_sender = ctx.factory.get_streams_by_sender(&ctx.sender);
    let by_recipient = ctx.factory.get_streams_by_recipient(&ctx.recipient);
    assert_eq!(by_sender, Vec::from_slice(&ctx.env, &[0]));
    assert_eq!(by_recipient, Vec::from_slice(&ctx.env, &[0]));
}

#[test]
fn test_deployed_stream_is_initialized_and_streamable() {
    let ctx = setup();
    let stream_address = create_stream(&ctx);

    // Inspect the deployed contract directly and verify its live behavior.
    let stream = stream_wasm::Client::new(&ctx.env, &stream_address);
    let info = stream.get_info();
    assert_eq!(info.sender, ctx.sender);
    assert_eq!(info.recipient, ctx.recipient);
    assert_eq!(info.total_amount, AMOUNT);
    assert_eq!(info.status, stream_wasm::StreamStatus::Active);

    // Advance 10 seconds → 1_000 streamed and withdrawable by the recipient.
    ctx.env.ledger().with_mut(|l| l.timestamp += 10);
    assert_eq!(stream.get_balance(), 1_000);

    let withdrawn = stream.withdraw(&ctx.recipient);
    assert_eq!(withdrawn, 1_000);
    assert_eq!(ctx.token.balance(&ctx.recipient), 1_000);
}

#[test]
fn test_create_stream_zero_amount_errors() {
    let ctx = setup();
    assert_eq!(
        ctx.factory.try_create_stream(
            &ctx.sender,
            &ctx.recipient,
            &ctx.token_address,
            &0,
            &DURATION
        ),
        Err(Ok(StreamError::ZeroAmount))
    );
    // Nothing was deployed in the failed attempt.
    assert_eq!(
        ctx.factory.try_get_stream(&0u32),
        Err(Ok(StreamError::StreamNotFound))
    );
}

#[test]
fn test_create_stream_zero_duration_errors() {
    let ctx = setup();
    assert_eq!(
        ctx.factory
            .try_create_stream(&ctx.sender, &ctx.recipient, &ctx.token_address, &AMOUNT, &0),
        Err(Ok(StreamError::InvalidFlowRate))
    );
    assert_eq!(
        ctx.factory.try_get_stream(&0u32),
        Err(Ok(StreamError::StreamNotFound))
    );
}

#[test]
fn test_create_stream_requires_sender_auth() {
    // No mocked auths: the sender must authorize the call (and the token
    // transfer inside it) for real, and doesn't.
    let ctx = setup_with_auth(false);
    assert!(ctx
        .factory
        .try_create_stream(
            &ctx.sender,
            &ctx.recipient,
            &ctx.token_address,
            &AMOUNT,
            &DURATION
        )
        .is_err());
}

#[test]
fn test_raw_initialize_call_decodes_stream_error() {
    let ctx = setup();
    let stream_address = create_stream(&ctx);

    // Re-run `initialize` on the now-initialized stream through the exact raw
    // `try_invoke_contract` path `deploy_stream` uses. The stream's own contract
    // error must come back as a real `StreamError` (AlreadyInitialized), not a
    // generic host trap or a silent no-op.
    let factory_addr = Address::generate(&ctx.env);
    let mut args: Vec<Val> = Vec::new(&ctx.env);
    args.push_back(factory_addr.into_val(&ctx.env));
    args.push_back(ctx.sender.clone().into_val(&ctx.env));
    args.push_back(ctx.recipient.clone().into_val(&ctx.env));
    args.push_back(ctx.token_address.clone().into_val(&ctx.env));
    args.push_back(FLOW_RATE.into_val(&ctx.env));
    args.push_back(AMOUNT.into_val(&ctx.env));
    args.push_back(0u64.into_val(&ctx.env));

    let result = ctx.env.try_invoke_contract::<(), StreamError>(
        &stream_address,
        &Symbol::new(&ctx.env, "initialize"),
        args,
    );
    assert_eq!(result, Err(Ok(StreamError::AlreadyInitialized)));
}

#[test]
fn test_create_stream_transfer_failure_aborts_whole_call() {
    // Sender is funded with less than the requested amount, so the escrow
    // transfer inside create_stream fails *after* deploy + initialize succeed.
    // Proves the transfer strictly follows a successful init, and that the
    // whole call fails without registering or moving anything.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_address = env.register(MockToken, ());
    let token = MockTokenClient::new(&env, &token_address);
    token.initialize(&admin);
    token.mint(&admin, &sender, &(AMOUNT / 2));

    let wasm_hash = env.deployer().upload_contract_wasm(stream_wasm::WASM);
    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(&env, &factory_id);
    factory.initialize(&admin, &wasm_hash);

    let created =
        factory.try_create_stream(&sender, &recipient, &token_address, &AMOUNT, &DURATION);
    assert!(created.is_err());

    // No escrow moved, and nothing was cached or indexed.
    assert_eq!(token.balance(&sender), AMOUNT / 2);
    assert_eq!(
        factory.try_get_stream(&0u32),
        Err(Ok(StreamError::StreamNotFound))
    );
    assert_eq!(factory.get_streams_by_sender(&sender).len(), 0);
    assert_eq!(factory.get_streams_by_recipient(&recipient).len(), 0);
}

#[test]
fn test_create_stream_deploy_failure_aborts_whole_call() {
    // Factory points at a wasm hash that was never uploaded: deployment fails
    // before any initialize runs, so the whole create_stream call must abort
    // and no escrow can move.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_address = env.register(MockToken, ());
    let token = MockTokenClient::new(&env, &token_address);
    token.initialize(&admin);
    token.mint(&admin, &sender, &(AMOUNT * 10));

    let bogus_hash = BytesN::from_array(&env, &[0xABu8; 32]);
    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(&env, &factory_id);
    factory.initialize(&admin, &bogus_hash);

    let created = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        factory.create_stream(&sender, &recipient, &token_address, &AMOUNT, &DURATION)
    }));
    assert!(
        created.is_err(),
        "create_stream must abort when deployment fails"
    );

    // No escrow moved, and nothing was cached.
    assert_eq!(token.balance(&sender), AMOUNT * 10);
    assert_eq!(
        factory.try_get_stream(&0u32),
        Err(Ok(StreamError::StreamNotFound))
    );
}

#[test]
fn test_multiple_streams_index_independently() {
    let ctx = setup();
    let other_recipient = Address::generate(&ctx.env);

    let s0 = create_stream(&ctx);
    ctx.factory.create_stream(
        &ctx.sender,
        &other_recipient,
        &ctx.token_address,
        &AMOUNT,
        &DURATION,
    );
    let s2 = create_stream(&ctx);

    let by_sender = ctx.factory.get_streams_by_sender(&ctx.sender);
    assert_eq!(by_sender, Vec::from_slice(&ctx.env, &[0, 1, 2]));
    assert_eq!(by_sender.len(), 3);

    // The middle stream belongs to the other recipient.
    assert_eq!(
        ctx.factory.get_streams_by_recipient(&other_recipient),
        Vec::from_slice(&ctx.env, &[1])
    );
    let info1 = ctx.factory.get_stream(&1);
    assert_eq!(info1.recipient, other_recipient);

    // Streams get distinct deterministic addresses for distinct IDs.
    assert_ne!(s0, s2);
    assert_eq!(ctx.token.balance(&s0), AMOUNT);
    assert_eq!(ctx.token.balance(&s2), AMOUNT);
}
