#![no_std]

#[allow(dead_code)]
mod events;
#[allow(dead_code)]
mod math;
#[allow(dead_code)]
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};
use trickle_common::MultistreamError;

// ═══════════════════════════════════════════════════════════════════════════════
// Multistream Contract
// ═══════════════════════════════════════════════════════════════════════════════

/// A single stream that splits payments proportionally across multiple
/// recipients based on assigned weights.
#[contract]
pub struct MultiStreamContract;

#[contractimpl]
impl MultiStreamContract {
    /// Initialize a multi-recipient stream.
    pub fn initialize(
        _env: Env,
        sender: Address,
        asset: Address,
        total_amount: i128,
        duration: u32,
    ) -> Result<(), MultistreamError> {
        let _ = (sender, asset, total_amount, duration);
        todo!("initialize multistream")
    }

    /// Add a recipient with a proportional weight.
    pub fn add_recipient(
        _env: Env,
        sender: Address,
        recipient: Address,
        weight: u32,
    ) -> Result<(), MultistreamError> {
        sender.require_auth();
        let _ = (sender, recipient, weight);
        todo!("add recipient")
    }

    /// Remove a recipient. Their accrued share is returned to sender.
    pub fn remove_recipient(
        _env: Env,
        sender: Address,
        recipient: Address,
    ) -> Result<(), MultistreamError> {
        sender.require_auth();
        let _ = (sender, recipient);
        todo!("remove recipient")
    }

    /// Withdraw accrued funds for a recipient.
    pub fn withdraw(_env: Env, recipient: Address) -> Result<i128, MultistreamError> {
        recipient.require_auth();
        let _ = recipient;
        todo!("implement multistream withdraw")
    }

    /// Pause the multistream (sender only).
    pub fn pause(_env: Env, sender: Address) -> Result<(), MultistreamError> {
        sender.require_auth();
        let _ = sender;
        todo!("pause multistream")
    }

    /// Resume the multistream after pausing (sender only).
    pub fn resume(_env: Env, sender: Address) -> Result<(), MultistreamError> {
        sender.require_auth();
        let _ = sender;
        todo!("resume multistream")
    }

    /// Cancel the multistream.
    pub fn cancel(_env: Env, sender: Address) -> Result<(), MultistreamError> {
        sender.require_auth();
        let _ = sender;
        todo!("cancel multistream")
    }

    /// Read-only: get current claimable for a recipient.
    pub fn get_claimable(_env: Env, recipient: Address) -> Result<i128, MultistreamError> {
        let _ = recipient;
        todo!("get claimable amount for multistream recipient")
    }

    /// Read-only: get flow rate per weight unit.
    pub fn get_flow_rate(_env: Env) -> Result<i128, MultistreamError> {
        todo!("get multistream flow rate")
    }
}

mod test;
