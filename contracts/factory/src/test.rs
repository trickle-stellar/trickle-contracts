#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, BytesN, Env};

fn setup() -> (Env, Address, FactoryContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(FactoryContract, ());
    let client = FactoryContractClient::new(&env, &contract_id);

    // Create a dummy WASM hash for testing.
    let wasm_hash = BytesN::from_array(&env, &[0u8; 32]);

    client.initialize(&admin, &wasm_hash);

    (env, admin, client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
    let contract_id = env.register(FactoryContract, ());
    let client = FactoryContractClient::new(&env, &contract_id);

    client.initialize(&admin, &wasm_hash);

    // After initialization, stream count should be 0.
    let streams = client.get_streams_by_sender(&Address::generate(&env));
    assert_eq!(streams.len(), 0);
}

#[test]
fn test_get_streams_empty() {
    let (env, _admin, client) = setup();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let sender_streams = client.get_streams_by_sender(&sender);
    let recipient_streams = client.get_streams_by_recipient(&recipient);

    assert_eq!(sender_streams.len(), 0);
    assert_eq!(recipient_streams.len(), 0);
}
