#![no_std]
#![allow(clippy::too_many_arguments)]

#[allow(dead_code)]
mod deploy;
mod events;
#[allow(dead_code)]
mod storage;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};
use trickle_common::{StreamError, StreamInfo};

// ═══════════════════════════════════════════════════════════════════════════════
// Factory Contract
// ═══════════════════════════════════════════════════════════════════════════════

/// The factory contract is the entry point for creating streams.
///
/// It deploys new stream contract instances, maintains a registry of
/// all streams, and provides indexing by sender/recipient address.
#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    /// Initialize the factory. Must be called once after deployment.
    ///
    /// # Arguments
    /// * `admin` - The address that controls factory settings.
    /// * `stream_wasm_hash` - The pre-uploaded WASM hash of the stream
    ///   contract. Upload with `stellar contract upload` before calling this.
    pub fn initialize(env: Env, admin: Address, stream_wasm_hash: BytesN<32>) {
        storage::set_admin(&env, &admin);
        storage::set_stream_wasm_hash(&env, &stream_wasm_hash);
        storage::set_stream_count(&env, &0u32);
    }

    /// Create a new payment stream.
    ///
    /// Deploys a new stream contract instance, transfers tokens from the
    /// sender into escrow, and registers the stream in the factory's
    /// index.
    ///
    /// # Arguments
    /// * `sender` - The address funding the stream (must authorize this call).
    /// * `recipient` - The address that will receive streamed funds.
    /// * `asset` - The Stellar token contract address (XLM, USDC, etc.).
    /// * `amount` - Total tokens to deposit into escrow (must be > 0).
    /// * `duration` - Stream duration in seconds (must be > 0).
    ///
    /// # Returns
    /// The address of the newly deployed stream contract.
    ///
    /// # Expected behavior
    /// 1. Require auth from sender.
    /// 2. Validate: amount > 0, duration > 0, sender != recipient.
    /// 3. Compute flow_rate = amount / duration.
    /// 4. Transfer `amount` from sender to this factory (temporary escrow).
    /// 5. Deploy a new stream contract via `deploy::deploy_stream`.
    /// 6. Cache StreamInfo in the registry.
    /// 7. Index the stream by sender and recipient.
    /// 8. Emit `stream_deployed` event.
    /// 9. Return the stream contract address.
    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        asset: Address,
        amount: i128,
        duration: u32,
    ) -> Result<Address, StreamError> {
        sender.require_auth();

        if amount <= 0 {
            return Err(StreamError::ZeroAmount);
        }
        if duration == 0 {
            return Err(StreamError::InvalidFlowRate);
        }

        let stream_id = storage::get_stream_count(&env);
        let wasm_hash = storage::get_stream_wasm_hash(&env);
        let flow_rate = amount / duration as i128;
        let current_time = env.ledger().timestamp();
        let factory_address = env.current_contract_address();

        // Deploy the new stream contract.
        let stream_address = deploy::deploy_stream(
            &env,
            &wasm_hash,
            &factory_address,
            &sender,
            &recipient,
            &asset,
            flow_rate,
            amount,
            current_time,
        )?;

        // Cache stream metadata in the registry.
        let info = StreamInfo {
            sender: sender.clone(),
            recipient: recipient.clone(),
            asset,
            flow_rate,
            total_amount: amount,
            withdrawn_amount: 0,
            start_time: current_time,
            last_update_time: current_time,
            status: trickle_common::StreamStatus::Active,
        };
        storage::set_stream_info(&env, stream_id, &info);

        // Index by sender and recipient.
        storage::add_stream_for_sender(&env, &sender, stream_id);
        storage::add_stream_for_recipient(&env, &recipient, stream_id);

        // Increment counter.
        storage::set_stream_count(&env, &(stream_id + 1));

        events::stream_deployed(&env, stream_id, &stream_address, &sender, &recipient);

        Ok(stream_address)
    }

    /// Read-only: get cached stream metadata by stream ID.
    pub fn get_stream(env: Env, stream_id: u32) -> Result<StreamInfo, StreamError> {
        storage::get_stream_info(&env, stream_id).ok_or(StreamError::StreamNotFound)
    }

    /// Read-only: get all stream IDs created by a given sender.
    pub fn get_streams_by_sender(env: Env, sender: Address) -> Vec<u32> {
        storage::get_streams_by_sender(&env, &sender)
    }

    /// Read-only: get all stream IDs receiving funds for a given recipient.
    pub fn get_streams_by_recipient(env: Env, recipient: Address) -> Vec<u32> {
        storage::get_streams_by_recipient(&env, &recipient)
    }
}

mod test;
