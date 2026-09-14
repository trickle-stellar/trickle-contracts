#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, testutils::Ledger, Address, Env,
};

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
    stream: StreamContractClient<'static>,
    token: MockTokenClient<'static>,
    sender: Address,
    recipient: Address,
    stream_address: Address,
}

// flow_rate 100/sec, total 10_000 — fully accrues in 100 seconds.
const FLOW_RATE: i128 = 100;
const TOTAL: i128 = 10_000;

fn setup() -> TestContext {
    setup_with_auth(true)
}

fn setup_with_auth(mock_auth: bool) -> TestContext {
    let env = Env::default();
    if mock_auth {
        env.mock_all_auths();
    }

    let admin = Address::generate(&env);
    let factor = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_id = env.register(MockToken, ());
    let token = MockTokenClient::new(&env, &token_id);
    token.initialize(&admin);

    let stream_address = env.register(StreamContract, ());
    let stream = StreamContractClient::new(&env, &stream_address);

    // Fund the stream's escrow directly (the factory does this on create).
    if mock_auth {
        token.mint(&admin, &stream_address, &TOTAL);
    }

    stream.initialize(
        &factor, &sender, &recipient, &token_id, &FLOW_RATE, &TOTAL, &0,
    );

    TestContext {
        env,
        stream,
        token,
        sender,
        recipient,
        stream_address,
    }
}

fn advance(ctx: &TestContext, seconds: u64) {
    ctx.env.ledger().with_mut(|l| l.timestamp += seconds);
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let ctx = setup();
    let info = ctx.stream.get_info();
    assert_eq!(info.sender, ctx.sender);
    assert_eq!(info.recipient, ctx.recipient);
    assert_eq!(info.asset, ctx.token.address);
    assert_eq!(info.flow_rate, FLOW_RATE);
    assert_eq!(info.total_amount, TOTAL);
    assert_eq!(info.withdrawn_amount, 0);
    assert_eq!(info.status, StreamStatus::Active);
}

#[test]
fn test_get_balance_after_creation() {
    let ctx = setup();
    assert_eq!(ctx.stream.get_balance(), 0);
}

#[test]
fn test_withdraw_accrues() {
    let ctx = setup();
    advance(&ctx, 10);

    let amount = ctx.stream.withdraw(&ctx.recipient);
    assert_eq!(amount, 1_000);

    assert_eq!(ctx.token.balance(&ctx.recipient), 1_000);
    assert_eq!(ctx.token.balance(&ctx.stream_address), TOTAL - 1_000);

    let info = ctx.stream.get_info();
    assert_eq!(info.withdrawn_amount, 1_000);
    assert_eq!(info.status, StreamStatus::Active);
}

#[test]
fn test_withdraw_caps_at_total() {
    let ctx = setup();
    advance(&ctx, 10_000);

    let amount = ctx.stream.withdraw(&ctx.recipient);
    assert_eq!(amount, TOTAL);
    assert_eq!(ctx.token.balance(&ctx.recipient), TOTAL);
    assert_eq!(ctx.token.balance(&ctx.stream_address), 0);
}

#[test]
fn test_withdraw_nothing_to_withdraw() {
    let ctx = setup();
    assert_eq!(
        ctx.stream.try_withdraw(&ctx.recipient),
        Err(Ok(StreamError::NothingToWithdraw))
    );
}

#[test]
fn test_withdraw_requires_recipient_auth() {
    // No mock auths: the caller must genuinely authorize.
    let ctx = setup_with_auth(false);
    let stranger = Address::generate(&ctx.env);
    assert!(ctx.stream.try_withdraw(&stranger).is_err());
}

#[test]
fn test_withdraw_wrong_recipient() {
    let ctx = setup();
    let stranger = Address::generate(&ctx.env);
    advance(&ctx, 10);

    assert_eq!(
        ctx.stream.try_withdraw(&stranger),
        Err(Ok(StreamError::Unauthorized))
    );
    // Nothing moved.
    assert_eq!(ctx.token.balance(&ctx.stream_address), TOTAL);
}

#[test]
fn test_pause_freezes_accrual() {
    let ctx = setup();
    advance(&ctx, 10);
    ctx.stream.pause(&ctx.sender);
    assert_eq!(ctx.stream.get_info().status, StreamStatus::Paused);

    // While paused, only the pre-pause 10 seconds have accrued.
    advance(&ctx, 50);
    assert_eq!(ctx.stream.get_balance(), 1_000);

    // The recipient can still withdraw the frozen amount.
    let amount = ctx.stream.withdraw(&ctx.recipient);
    assert_eq!(amount, 1_000);
}

#[test]
fn test_pause_resume_preserves_prepause_accrual() {
    let ctx = setup();
    advance(&ctx, 10); // accrue 1000
    ctx.stream.pause(&ctx.sender);
    advance(&ctx, 20); // paused: nothing new accrues
    ctx.stream.resume(&ctx.sender);

    advance(&ctx, 10); // post-resume: another 1000
    assert_eq!(ctx.stream.get_balance(), 2_000);
    assert_eq!(ctx.stream.get_info().status, StreamStatus::Active);
}

#[test]
fn test_withdraw_while_paused_then_resume_no_double_accrual() {
    let ctx = setup();
    advance(&ctx, 10); // accrue 1000
    ctx.stream.pause(&ctx.sender);

    advance(&ctx, 5); // paused: nothing new accrues
    assert_eq!(ctx.stream.get_balance(), 1_000); // frozen at pause

    // Withdraw the frozen amount while still paused (checkpoint bumps to paused_at).
    let amount = ctx.stream.withdraw(&ctx.recipient);
    assert_eq!(amount, 1_000);

    // Resume: the pre-pause accrual was already paid out, so the excluded
    // window (`paused_at - last_update_time`) is zero and the checkpoint
    // simply rolls forward to now — nothing is double counted.
    ctx.stream.resume(&ctx.sender);

    advance(&ctx, 10); // post-resume: exactly another 1000
    assert_eq!(ctx.stream.get_balance(), 1_000); // NOT 2000
    assert_eq!(ctx.stream.get_info().withdrawn_amount, 1_000);

    // Clean continuation: recipient ends with 2000 in total.
    let second = ctx.stream.withdraw(&ctx.recipient);
    assert_eq!(second, 1_000);
    assert_eq!(ctx.token.balance(&ctx.recipient), 2_000);
}

#[test]
fn test_pause_double_pause_errors() {
    let ctx = setup();
    ctx.stream.pause(&ctx.sender);
    assert_eq!(
        ctx.stream.try_pause(&ctx.sender),
        Err(Ok(StreamError::StreamNotActive))
    );
}

#[test]
fn test_resume_active_stream_errors() {
    let ctx = setup();
    assert_eq!(
        ctx.stream.try_resume(&ctx.sender),
        Err(Ok(StreamError::StreamNotPaused))
    );
}

#[test]
fn test_resume_double_resume_errors() {
    let ctx = setup();
    ctx.stream.pause(&ctx.sender);
    ctx.stream.resume(&ctx.sender);
    assert_eq!(
        ctx.stream.try_resume(&ctx.sender),
        Err(Ok(StreamError::StreamNotPaused))
    );
}

#[test]
fn test_pause_or_resume_unauthorized() {
    let ctx = setup();
    let stranger = Address::generate(&ctx.env);
    assert_eq!(
        ctx.stream.try_pause(&stranger),
        Err(Ok(StreamError::Unauthorized))
    );
    assert_eq!(
        ctx.stream.try_resume(&stranger),
        Err(Ok(StreamError::Unauthorized))
    );
}

#[test]
fn test_cancel_splits_funds() {
    let ctx = setup();
    advance(&ctx, 10); // accrued 1000, remaining escrow 10_000

    ctx.stream.cancel(&ctx.sender);

    // Recipient keeps the accrued 1000; sender gets the 9000 refund.
    assert_eq!(ctx.token.balance(&ctx.recipient), 1_000);
    assert_eq!(ctx.token.balance(&ctx.sender), 9_000);
    assert_eq!(ctx.token.balance(&ctx.stream_address), 0);

    let info = ctx.stream.get_info();
    assert_eq!(info.status, StreamStatus::Cancelled);
    assert_eq!(info.withdrawn_amount, TOTAL);

    // Withdrawing from a cancelled stream fails.
    assert_eq!(
        ctx.stream.try_withdraw(&ctx.recipient),
        Err(Ok(StreamError::StreamNotActive))
    );
    // Cancelling a second time fails.
    assert_eq!(
        ctx.stream.try_cancel(&ctx.sender),
        Err(Ok(StreamError::StreamNotActive))
    );
}

#[test]
fn test_cancel_immediately_refunds_all() {
    let ctx = setup();
    ctx.stream.cancel(&ctx.sender);

    assert_eq!(ctx.token.balance(&ctx.sender), TOTAL);
    assert_eq!(ctx.token.balance(&ctx.recipient), 0);
    assert_eq!(ctx.stream.get_info().status, StreamStatus::Cancelled);
}

#[test]
fn test_completed_status_after_full_vest() {
    let ctx = setup();
    advance(&ctx, 10_000);
    ctx.stream.withdraw(&ctx.recipient);

    // Derived status: fully withdrawn while still Active => Completed.
    assert_eq!(ctx.stream.get_info().status, StreamStatus::Completed);
}
