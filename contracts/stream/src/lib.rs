#![no_std]
#![allow(clippy::too_many_arguments)]

mod events;
#[allow(dead_code)]
mod math;
#[allow(dead_code)]
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};
use trickle_common::{StreamError, StreamInfo};

use storage::StreamConfig;

// ═══════════════════════════════════════════════════════════════════════════════
// Stream Contract
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-stream contract instance. Deployed by the factory.
///
/// Each stream contract holds escrowed tokens and manages its own
/// accrual, withdrawal, pause/resume, and cancellation logic.
#[contract]
pub struct StreamContract;

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl StreamContract {
    /// Initialize this stream contract. Called once by the factory
    /// immediately after deployment.
    ///
    /// Stores the stream configuration and marks the contract as initialized.
    /// Tokens should already be held in escrow by the factory or transferred
    /// to this contract during deployment.
    pub fn initialize(
        env: Env,
        factory: Address,
        sender: Address,
        recipient: Address,
        asset: Address,
        flow_rate: i128,
        total_amount: i128,
        start_time: u64,
    ) -> Result<(), StreamError> {
        if storage::is_initialized(&env) {
            return Err(StreamError::AlreadyInitialized);
        }

        let config = StreamConfig {
            factory: factory.clone(),
            sender: sender.clone(),
            recipient: recipient.clone(),
            asset,
            flow_rate,
            total_amount,
            withdrawn_amount: 0,
            start_time,
            last_update_time: start_time,
        };

        storage::set_config(&env, &config);
        storage::set_initialized(&env);
        events::stream_initialized(&env, &factory, &sender, &recipient);

        Ok(())
    }

    /// Withdraw accrued funds from this stream.
    ///
    /// Only the recipient can call this. Calculates how much has streamed
    /// since the last update, transfers tokens, and updates state.
    ///
    /// # Expected behavior
    /// 1. Require auth from the stream's recipient.
    /// 2. Load config, verify status is Active.
    /// 3. Calculate claimable = `math::calculate_claimable`.
    /// 4. If claimable <= 0, return `NothingToWithdraw`.
    /// 5. Transfer claimable from this contract to recipient via token client.
    /// 6. Update config: withdrawn_amount += claimable, last_update_time = now.
    /// 7. Emit `withdrawn` event.
    /// 8. Return claimable.
    pub fn withdraw(env: Env, recipient: Address) -> Result<i128, StreamError> {
        recipient.require_auth();

        let mut config = storage::get_config(&env);

        if recipient != config.recipient {
            return Err(StreamError::Unauthorized);
        }

        let claimable = math::calculate_claimable(&config, env.ledger().timestamp());

        if claimable <= 0 {
            return Err(StreamError::NothingToWithdraw);
        }

        // TODO: Transfer tokens to recipient.
        //   let token = soroban_sdk::token::Client::new(&env, &config.asset);
        //   token.transfer(&env.current_contract_address(), &config.recipient, &claimable);

        config.withdrawn_amount += claimable;
        config.last_update_time = env.ledger().timestamp();
        storage::set_config(&env, &config);

        events::withdrawn(&env, &config.recipient, claimable);

        Ok(claimable)
    }

    /// Pause an active stream. Only the sender (funder) can pause.
    ///
    /// While paused, no new funds accrue. The stream can be resumed later.
    pub fn pause(env: Env, sender: Address) -> Result<(), StreamError> {
        sender.require_auth();

        let mut config = storage::get_config(&env);

        if sender != config.sender {
            return Err(StreamError::Unauthorized);
        }

        // TODO: Check status is Active (need to store/derive status).
        // For now, status tracking is a placeholder in math::derive_status.

        config.last_update_time = env.ledger().timestamp();
        storage::set_config(&env, &config);

        events::paused(&env, &sender);

        Ok(())
    }

    /// Resume a paused stream. Only the sender (funder) can resume.
    ///
    /// Resets the last_update_time to the current ledger time so that
    /// paused duration is excluded from accrual.
    pub fn resume(env: Env, sender: Address) -> Result<(), StreamError> {
        sender.require_auth();

        let mut config = storage::get_config(&env);

        if sender != config.sender {
            return Err(StreamError::Unauthorized);
        }

        // TODO: Check status is Paused.

        config.last_update_time = env.ledger().timestamp();
        storage::set_config(&env, &config);

        events::resumed(&env, &sender);

        Ok(())
    }

    /// Cancel an active or paused stream. Only the sender (funder) can cancel.
    ///
    /// Accrued but unwithdrawn funds go to the recipient (they earned it).
    /// Remaining unstreamed funds are refunded to the sender.
    ///
    /// # Expected behavior
    /// 1. Require auth from sender.
    /// 2. Calculate claimable (accrued but unwithdrawn).
    /// 3. Transfer claimable to recipient.
    /// 4. Calculate refund = total_amount - withdrawn_amount - claimable.
    /// 5. Transfer refund back to sender.
    /// 6. Set status to Cancelled.
    /// 7. Emit `cancelled` event.
    pub fn cancel(env: Env, sender: Address) -> Result<(), StreamError> {
        sender.require_auth();

        let mut config = storage::get_config(&env);

        if sender != config.sender {
            return Err(StreamError::Unauthorized);
        }

        let claimable = math::calculate_claimable(&config, env.ledger().timestamp());
        let remaining = math::calculate_remaining(&config);
        let refund = remaining - claimable;

        // TODO: Transfer claimable to recipient.
        // TODO: Transfer refund to sender.

        config.withdrawn_amount = config.total_amount;
        config.last_update_time = env.ledger().timestamp();
        storage::set_config(&env, &config);

        events::cancelled(&env, &sender, refund, claimable);

        Ok(())
    }

    /// Read-only: calculate the current claimable balance.
    ///
    /// Returns how much the recipient could withdraw right now.
    pub fn get_balance(env: Env) -> Result<i128, StreamError> {
        let config = storage::get_config(&env);
        Ok(math::calculate_claimable(&config, env.ledger().timestamp()))
    }

    /// Read-only: get the full stream metadata as StreamInfo.
    pub fn get_info(env: Env) -> Result<StreamInfo, StreamError> {
        let config = storage::get_config(&env);
        Ok(storage::config_to_info(&config))
    }

    /// Called only by the authorized stream-nft contract to transfer
    /// stream receivership to a new address when the NFT is transferred.
    ///
    /// # Authorization
    /// Requires auth from the stored NftContract address. Only that
    /// contract may call this function.
    ///
    /// # Expected behavior
    /// 1. Load the authorized NftContract address from storage.
    /// 2. Require auth from that address.
    /// 3. Update config.recipient to new_recipient.
    /// 4. Emit `recipient_updated` event.
    pub fn update_recipient(env: Env, new_recipient: Address) -> Result<(), StreamError> {
        let nft_contract = storage::get_nft_contract(&env).ok_or(StreamError::Unauthorized)?;

        nft_contract.require_auth();

        let mut config = storage::get_config(&env);
        let old_recipient = config.recipient.clone();

        config.recipient = new_recipient.clone();
        storage::set_config(&env, &config);

        events::recipient_updated(&env, &old_recipient, &new_recipient);

        Ok(())
    }
}

mod test;
