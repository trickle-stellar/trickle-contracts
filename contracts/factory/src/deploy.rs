use soroban_sdk::{Address, BytesN, Env, IntoVal, Symbol, Val, Vec};
use trickle_common::StreamError;

/// Deploy a new stream contract instance.
///
/// Uses the factory's stored WASM hash to deploy a new Soroban contract,
/// then calls `initialize` on the newly deployed contract.
///
/// The new contract's address is deterministic: it is derived from this
/// factory's address plus a salt built from `stream_id`, so a given
/// stream ID always maps to a given contract address. Because the factory
/// address is part of the derivation, two factories never collide.
///
/// `initialize` is invoked through the raw `try_invoke_contract` API instead
/// of a generated client: linking the `trickle-stream` contract crate as a
/// library would pull that contract's exported entrypoints into the factory's
/// WASM (duplicate `initialize` symbol during LTO), so we call it by symbol.
///
/// The invocation is still fully typed on the way out: the child's `Val`
/// return is decoded into `Result<(), StreamError>` (contract errors are
/// re-trapped by the SDK and mapped back by `StreamError: TryFrom<Error>`).
/// On failure the whole `create_stream` call short-circuits here, before any
/// escrow transfer can move, and the failed transaction reverts the deploy —
/// so a bad init can never be a silent no-op or leave funds stranded.
///
/// # Arguments
/// * `env` - The Soroban environment.
/// * `wasm_hash` - The pre-uploaded WASM bytecode hash of the stream contract.
/// * `factory` - Address of this factory contract (passed to stream's initialize).
/// * `sender` - The address funding the stream.
/// * `recipient` - The address that will receive streamed funds.
/// * `asset` - The Stellar token contract address.
/// * `flow_rate` - Per-second token flow rate.
/// * `total_amount` - Total tokens to escrow.
/// * `start_time` - Ledger timestamp when streaming begins.
/// * `stream_id` - Monotonic stream ID; derives the deterministic deploy salt.
///
/// # Returns
/// The address of the newly deployed stream contract.
pub fn deploy_stream(
    env: &Env,
    wasm_hash: &BytesN<32>,
    factory: &Address,
    sender: &Address,
    recipient: &Address,
    asset: &Address,
    flow_rate: i128,
    total_amount: i128,
    start_time: u64,
    stream_id: u32,
) -> Result<Address, StreamError> {
    // Salt = 32 bytes, last 4 = big-endian stream ID. The factory address is
    // already folded in by `with_current_contract`, so sequential IDs produce
    // well-spaced deterministic addresses without cross-factory collisions.
    let mut salt_bytes = [0u8; 32];
    salt_bytes[28..].copy_from_slice(&stream_id.to_be_bytes());
    let salt = BytesN::from_array(env, &salt_bytes);

    let stream_address = env
        .deployer()
        .with_current_contract(salt)
        .deploy_v2(wasm_hash.clone(), ());

    // `StreamContract::initialize(factory, sender, recipient, asset, flow_rate, total_amount, start_time)`.
    let mut args: Vec<Val> = Vec::new(env);
    args.push_back(factory.clone().into_val(env));
    args.push_back(sender.clone().into_val(env));
    args.push_back(recipient.clone().into_val(env));
    args.push_back(asset.clone().into_val(env));
    args.push_back(flow_rate.into_val(env));
    args.push_back(total_amount.into_val(env));
    args.push_back(start_time.into_val(env));

    match env.try_invoke_contract::<(), StreamError>(
        &stream_address,
        &Symbol::new(env, "initialize"),
        args,
    ) {
        Ok(Ok(())) => Ok(stream_address),
        Ok(Err(_)) | Err(Err(_)) => {
            // A freshly deployed contract can never be already-initialized and
            // `initialize` requires no auth, so these branches only guard
            // against unexpected host/diagnostic failures.
            Err(StreamError::Unauthorized)
        }
        Err(Ok(stream_error)) => Err(stream_error),
    }
}
