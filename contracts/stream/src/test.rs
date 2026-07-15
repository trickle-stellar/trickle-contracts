#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

// ─── Test Setup ─────────────────────────────────────────────────────────────

fn setup() -> (Env, Address, StreamContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let contract_id = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &contract_id);

    let factory = Address::generate(&env);
    let recipient = Address::generate(&env);
    let asset = Address::generate(&env);

    client.initialize(
        &factory, &sender, &recipient, &asset, &100,    // flow_rate: 100 tokens/sec
        &10_000, // total_amount
        &0,      // start_time
    );

    (env, sender, client)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let factory = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let asset = Address::generate(&env);
    let contract_id = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &contract_id);

    client.initialize(&factory, &sender, &recipient, &asset, &100, &10_000, &0);

    let info = client.get_info();
    assert_eq!(info.sender, sender);
    assert_eq!(info.recipient, recipient);
    assert_eq!(info.asset, asset);
    assert_eq!(info.flow_rate, 100);
    assert_eq!(info.total_amount, 10_000);
    assert_eq!(info.withdrawn_amount, 0);
}

#[test]
fn test_get_balance_after_creation() {
    let (_env, _sender, client) = setup();

    // Immediately after creation (same ledger time), balance should be 0.
    let balance = client.get_balance();
    assert_eq!(balance, 0);
}

// ─── More tests for contributors to implement ───────────────────────────────
//
// #[test]
// fn test_withdraw_after_time() {
//     let (env, _sender, client) = setup();
//     // Advance ledger by 10 seconds.
//     env.ledger().with_mut(|l| l.timestamp = l.timestamp + 10);
//     // Balance should be 100 * 10 = 1000.
//     let balance = client.get_balance();
//     assert_eq!(balance, 1_000);
// }
//
// #[test]
// fn test_update_recipient_unauthorized() {
//     let (env, _sender, client) = setup();
//     let stranger = Address::generate(&env);
//     let result = client.try_update_recipient(&stranger);
//     assert!(result.is_err());
// }
