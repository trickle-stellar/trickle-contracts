#![no_std]
#![allow(clippy::too_many_arguments)]

mod events;
mod math;
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};
use trickle_common::{VestingError, VestingInfo, VestingStatus};

// ═══════════════════════════════════════════════════════════════════════════════
// Vesting Contract
// ═══════════════════════════════════════════════════════════════════════════════

/// Time-locked token release. Tokens vest linearly after a cliff and can
/// be claimed by the beneficiary as they unlock.
#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    /// Create a new vesting contract.
    ///
    /// # Arguments
    /// * `admin` - The deployer (can revoke if revocable).
    /// * `beneficiary` - The address receiving vested tokens.
    /// * `asset` - Token contract address.
    /// * `total_amount` - Total tokens to vest.
    /// * `cliff_duration` - Cliff period in seconds (no tokens before cliff).
    /// * `vesting_duration` - Total vesting duration in seconds.
    /// * `revocable` - Whether admin can revoke.
    pub fn initialize(
        env: Env,
        admin: Address,
        beneficiary: Address,
        asset: Address,
        total_amount: i128,
        cliff_duration: u64,
        vesting_duration: u64,
        revocable: bool,
    ) -> Result<(), VestingError> {
        if total_amount <= 0 {
            return Err(VestingError::NothingToVest);
        }
        if vesting_duration <= cliff_duration {
            return Err(VestingError::NothingToVest);
        }

        let start_time = env.ledger().timestamp();

        let config = storage::VestingConfig {
            beneficiary: beneficiary.clone(),
            asset,
            total_amount,
            start_time,
            cliff_duration,
            vesting_duration,
            revocable,
            revocation_time: None,
        };

        storage::set_admin(&env, &admin);
        storage::set_config(&env, &config);
        storage::set_claimed(&env, &0i128);

        events::vesting_initialized(&env, &beneficiary, total_amount);

        Ok(())
    }

    /// Claim all currently vested tokens.
    ///
    /// # Expected behavior
    /// 1. Calculate vested from elapsed time.
    /// 2. Subtract already claimed.
    /// 3. Transfer claimable to beneficiary.
    /// 4. Update claimed amount.
    /// 5. Emit `vested_claimed` event.
    pub fn claim(env: Env, beneficiary: Address) -> Result<i128, VestingError> {
        beneficiary.require_auth();

        let config = storage::get_config(&env);
        let claimed = storage::get_claimed(&env);

        if beneficiary != config.beneficiary {
            return Err(VestingError::Unauthorized);
        }

        let elapsed = env.ledger().timestamp() - config.start_time;

        let claimable = math::calculate_claimable(
            config.total_amount,
            elapsed,
            config.cliff_duration,
            config.vesting_duration,
            claimed,
        )?;

        if claimable <= 0 {
            return Ok(0);
        }

        // TODO: Transfer claimable from contract to beneficiary via token client
        //   let token = soroban_sdk::token::Client::new(&env, &config.asset);
        //   token.transfer(&env.current_contract_address(), &beneficiary, &claimable);

        storage::set_claimed(&env, &(claimed + claimable));
        events::vested_claimed(&env, &beneficiary, claimable);

        Ok(claimable)
    }

    /// Revoke a revocable vesting contract (admin only).
    ///
    /// # Expected behavior
    /// 1. Validate revocable == true and not already revoked.
    /// 2. Calculate unvested = total_amount - vested.
    /// 3. Transfer unvested back to admin.
    /// 4. Mark as revoked (set revocation_time).
    pub fn revoke(env: Env, admin: Address) -> Result<i128, VestingError> {
        admin.require_auth();

        if admin != storage::get_admin(&env) {
            return Err(VestingError::Unauthorized);
        }

        let mut config = storage::get_config(&env);

        if !config.revocable || config.revocation_time.is_some() {
            return Err(VestingError::Unauthorized);
        }

        let elapsed = env.ledger().timestamp() - config.start_time;
        let vested = math::calculate_vested(
            config.total_amount,
            elapsed,
            config.cliff_duration,
            config.vesting_duration,
        )?;

        let unvested = config.total_amount - vested;

        // TODO: Transfer unvested to admin via token client
        //   let token = soroban_sdk::token::Client::new(&env, &config.asset);
        //   token.transfer(&env.current_contract_address(), &admin, &unvested);

        config.revocation_time = Some(env.ledger().timestamp());
        storage::set_config(&env, &config);

        events::vesting_revoked(&env, &admin, unvested);

        Ok(unvested)
    }

    /// Read-only: get current vesting status.
    pub fn get_status(env: Env) -> Result<VestingInfo, VestingError> {
        let config = storage::get_config(&env);
        let _claimed = storage::get_claimed(&env);
        let elapsed = env.ledger().timestamp() - config.start_time;

        let vested = math::calculate_vested(
            config.total_amount,
            elapsed,
            config.cliff_duration,
            config.vesting_duration,
        )?;

        let status = if config.revocation_time.is_some() {
            VestingStatus::Revoked
        } else if vested >= config.total_amount {
            VestingStatus::Completed
        } else if elapsed < config.cliff_duration {
            VestingStatus::NotStarted
        } else {
            VestingStatus::Vesting
        };

        Ok(VestingInfo {
            beneficiary: config.beneficiary,
            asset: config.asset,
            total_amount: config.total_amount,
            vested_amount: vested,
            start_time: config.start_time,
            cliff_duration: config.cliff_duration,
            vesting_duration: config.vesting_duration,
            status,
        })
    }

    /// Read-only: get the vesting flow rate (tokens per second).
    pub fn get_flow_rate(env: Env) -> Result<i128, VestingError> {
        let config = storage::get_config(&env);

        let rate = config.total_amount / (config.vesting_duration - config.cliff_duration) as i128;

        Ok(rate)
    }
}

mod test;
