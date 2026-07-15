#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_stream_nft_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let stream = Address::generate(&env);

    let contract_id = env.register(StreamNftContract, ());
    let client = StreamNftContractClient::new(&env, &contract_id);

    client.initialize(&admin, &stream);

    let stored_stream = client.get_stream_contract();
    assert_eq!(stored_stream, stream);
}

#[test]
fn test_stream_nft_get_stream_contract_before_init() {
    let env = Env::default();
    let contract_id = env.register(StreamNftContract, ());
    let client = StreamNftContractClient::new(&env, &contract_id);

    let result = client.try_get_stream_contract();
    assert!(result.is_err()); // storage::get panics when not initialized
}
