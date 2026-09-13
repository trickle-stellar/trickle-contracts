#![no_std]
#![allow(clippy::too_many_arguments)]

mod events;
mod math;
mod storage;

use soroban_sdk::{contract, contractimpl, token, Address, Env};
use trickle_common::{StreamError, StreamInfo, StreamStatus};

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
            status: StreamStatus::Active,
            paused_at: None,
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
    /// 2. Load config, verify the stream is not cancelled.
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
        if config.status == StreamStatus::Cancelled {
            return Err(StreamError::StreamNotActive);
        }

        let current_time = env.ledger().timestamp();
        let claimable = math::calculate_claimable(&config, current_time);

        if claimable <= 0 {
            return Err(StreamError::NothingToWithdraw);
        }

        // Transfer the accrued tokens from this stream's escrow to the recipient.
        let token = token::Client::new(&env, &config.asset);
        token.transfer(
            &env.current_contract_address(),
            &config.recipient,
            &claimable,
        );

        // Advance the checkpoint to the effective time. While paused this is the
        // frozen pause timestamp, so the checkpoint never drifts through a paused window.
        config.withdrawn_amount += claimable;
        config.last_update_time = math::effective_time(&config, current_time);
        storage::set_config(&env, &config);

        events::withdrawn(&env, &config.recipient, claimable);

        Ok(claimable)
    }

    /// Pause an active stream. Only the sender (funder) can pause.
    ///
    /// While paused, no new funds accrue: accrual is frozen at the pause
    /// timestamp and the paused window is excluded entirely on resume.
    /// The stream can be resumed later.
    pub fn pause(env: Env, sender: Address) -> Result<(), StreamError> {
        sender.require_auth();

        let mut config = storage::get_config(&env);

        if sender != config.sender {
            return Err(StreamError::Unauthorized);
        }
        if math::derive_status(&config) != StreamStatus::Active {
            return Err(StreamError::StreamNotActive);
        }

        // Freeze accrual at the pause timestamp without moving the accrual
        // checkpoint, so the recipient keeps the amount accrued up to this point.
        config.status = StreamStatus::Paused;
        config.paused_at = Some(env.ledger().timestamp());
        storage::set_config(&env, &config);

        events::paused(&env, &sender);

        Ok(())
    }

    /// Resume a paused stream. Only the sender (funder) can resume.
    ///
    /// Rolls the accrual checkpoint back by the paused window, so the amount
    /// accrued before the pause is preserved while the paused duration itself
    /// remains excluded from accrual.
    pub fn resume(env: Env, sender: Address) -> Result<(), StreamError> {
        sender.require_auth();

        let mut config = storage::get_config(&env);

        if sender != config.sender {
            return Err(StreamError::Unauthorized);
        }
        if config.status != StreamStatus::Paused {
            return Err(StreamError::StreamNotPaused);
        }

        let now = env.ledger().timestamp();
        let paused_at = config.paused_at.expect("paused stream must have paused_at");
        let excluded = paused_at.saturating_sub(config.last_update_time);

        // Rewind the checkpoint by the excluded (paused) window. The pre-pause
        // accrual is retained, and nothing accrued during the pause.
        config.last_update_time = now.saturating_sub(excluded);
        config.status = StreamStatus::Active;
        config.paused_at = None;
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
        if config.status == StreamStatus::Cancelled
            || config.withdrawn_amount >= config.total_amount
        {
            return Err(StreamError::StreamNotActive);
        }

        let current_time = env.ledger().timestamp();
        let claimable = math::calculate_claimable(&config, current_time);
        let remaining = math::calculate_remaining(&config);
        let refund = remaining - claimable;

        // Settle the recipient's earned amount first...
        let token = token::Client::new(&env, &config.asset);
        token.transfer(
            &env.current_contract_address(),
            &config.recipient,
            &claimable,
        );
        // ...then refund the unstreamed remainder to the sender.
        if refund > 0 {
            token.transfer(&env.current_contract_address(), &config.sender, &refund);
        }

        // Mark cancelled *before* setting withdrawn_amount = total_amount so a
        // mid-transaction status read can never surface the derived `Completed`
        // override (which is gated on status == Active).
        config.status = StreamStatus::Cancelled;
        config.withdrawn_amount = config.total_amount;
        config.last_update_time = current_time;
        config.paused_at = None;
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
