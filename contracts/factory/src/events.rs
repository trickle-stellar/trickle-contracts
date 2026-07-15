use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a new stream contract is deployed by the factory.
pub fn stream_deployed(
    env: &Env,
    stream_id: u32,
    stream_address: &Address,
    sender: &Address,
    recipient: &Address,
) {
    env.events().publish(
        (symbol_short!("deploy"), stream_id, sender),
        (stream_address, recipient),
    );
}
