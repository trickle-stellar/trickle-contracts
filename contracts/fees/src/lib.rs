#![no_std]

#[allow(dead_code)]
mod events;
mod math;
#[allow(dead_code)]
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};
use trickle_common::FeeError;

// ═══════════════════════════════════════════════════════════════════════════════
// Fee Contract
// ═══════════════════════════════════════════════════════════════════════════════

/// Protocol fee collector. Calculates and collects fees on stream
/// creation and/or withdrawals.
#[contract]
pub struct FeeContract;

#[contractimpl]
impl FeeContract {
    /// Initialize the fee contract. Must be called once after deployment.
    ///
    /// # Arguments
    /// * `admin` - The address that can update fees and withdraw.
    /// * `fee_recipient` - The address where withdrawn fees are sent.
    pub fn initialize(env: Env, admin: Address, fee_recipient: Address) {
        storage::set_admin(&env, &admin);
        storage::set_fee_recipient(&env, &fee_recipient);
        storage::set_fee_rate(&env, &0u32);
    }

    /// Calculate the fee for a given amount using the configured fee rate.
    ///
    /// Pure function — no storage access, no side effects.
    /// Returns 0 if fee_rate is 0.
    pub fn calculate_fee(env: Env, amount: i128) -> Result<i128, FeeError> {
        let fee_rate = storage::get_fee_rate(&env);
        math::calculate_fee(amount, fee_rate)
    }

    /// Collect a fee on stream creation.
    ///
    /// # Arguments
    /// * `payer` - The address paying the fee (typically the stream sender).
    /// * `amount` - The stream creation amount (fee is calculated from this).
    ///
    /// # Returns
    /// The fee amount collected.
    ///
    /// # Expected behavior
    /// 1. Calculate fee = `calculate_fee(amount, fee_rate)`.
    /// 2. Transfer fee from payer to this contract via token client.
    /// 3. Add fee to accumulated fees.
    /// 4. Emit `fee_collected` event.
    /// 5. Return fee amount.
    pub fn collect_creation_fee(_env: Env, payer: Address, amount: i128) -> Result<i128, FeeError> {
        // TODO: Implement fee collection.
        //   let fee_rate = storage::get_fee_rate(&env);
        //   let fee = math::calculate_fee(amount, fee_rate)?;
        //   let token = soroban_sdk::token::Client::new(&env, &asset);
        //   token.transfer(&payer, &env.current_contract_address(), &fee);
        //   storage::add_accumulated_fees(&env, fee);
        //   events::fee_collected(&env, &payer, amount, fee);
        //   Ok(fee)
        let _ = (payer, amount);
        todo!("implement creation fee collection")
    }

    /// Collect a fee on withdrawal.
    ///
    /// # Arguments
    /// * `recipient` - The address receiving the withdrawal (fee deducted).
    /// * `amount` - The gross withdrawal amount.
    ///
    /// # Returns
    /// The fee amount collected.
    ///
    /// # Expected behavior
    /// 1. Calculate fee = `calculate_fee(amount, fee_rate)`.
    /// 2. Transfer (amount - fee) to recipient, fee stays in contract.
    /// 3. Add fee to accumulated fees.
    /// 4. Emit `fee_collected` event.
    /// 5. Return fee amount.
    pub fn collect_withdrawal_fee(
        _env: Env,
        recipient: Address,
        amount: i128,
    ) -> Result<i128, FeeError> {
        // TODO: Implement withdrawal fee collection.
        let _ = (recipient, amount);
        todo!("implement withdrawal fee collection")
    }

    /// Update the fee rate (admin only).
    ///
    /// # Arguments
    /// * `new_rate` - New fee rate in basis points (100 = 1%).
    pub fn set_fee_rate(env: Env, admin: Address, new_rate: u32) -> Result<(), FeeError> {
        admin.require_auth();

        if admin != storage::get_admin(&env) {
            return Err(FeeError::Unauthorized);
        }

        let old_rate = storage::get_fee_rate(&env);
        storage::set_fee_rate(&env, &new_rate);
        events::fee_rate_updated(&env, old_rate, new_rate);

        Ok(())
    }

    /// Read-only: get the total accumulated fees awaiting withdrawal.
    pub fn get_accumulated_fees(env: Env) -> i128 {
        storage::get_accumulated_fees(&env)
    }

    /// Withdraw accumulated fees to the fee recipient (admin only).
    ///
    /// # Expected behavior
    /// 1. Require auth from admin.
    /// 2. Load accumulated fees and fee recipient.
    /// 3. Transfer accumulated fees to fee recipient via token client.
    /// 4. Reset accumulated fees to 0.
    /// 5. Emit `fees_withdrawn` event.
    pub fn withdraw_fees(env: Env, admin: Address) -> Result<(), FeeError> {
        admin.require_auth();

        if admin != storage::get_admin(&env) {
            return Err(FeeError::Unauthorized);
        }

        // TODO: Implement fee withdrawal.
        //   let amount = storage::get_accumulated_fees(&env);
        //   let recipient = storage::get_fee_recipient(&env);
        //   let token = soroban_sdk::token::Client::new(&env, &asset);
        //   token.transfer(&env.current_contract_address(), &recipient, &amount);
        //   storage::reset_accumulated_fees(&env);
        //   events::fees_withdrawn(&env, &admin, amount, &recipient);

        todo!("implement fee withdrawal")
    }
}

mod test;
