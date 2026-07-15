#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

fn setup() -> (Env, Address, FeeContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let contract_id = env.register(FeeContract, ());
    let client = FeeContractClient::new(&env, &contract_id);

    client.initialize(&admin, &fee_recipient);

    (env, admin, client)
}

#[test]
fn test_calculate_fee() {
    let (_env, _admin, client) = setup();

    // Set fee rate to 250 basis points (2.5%).
    client.set_fee_rate(&_admin, &250);

    // 10_000 * 250 / 10_000 = 250
    let fee = client.calculate_fee(&10_000);
    assert_eq!(fee, 250);
}

#[test]
fn test_calculate_fee_zero_rate() {
    let (_env, _admin, client) = setup();

    // Fee rate is 0 by default.
    let fee = client.calculate_fee(&10_000);
    assert_eq!(fee, 0);
}

#[test]
fn test_set_fee_rate() {
    let (_env, admin, client) = setup();

    client.set_fee_rate(&admin, &500);

    let fee = client.calculate_fee(&10_000);
    assert_eq!(fee, 500); // 5%
}
