#![no_std]

#[allow(dead_code)]
mod events;
#[allow(dead_code)]
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};
use trickle_common::StreamNftError;

// ═══════════════════════════════════════════════════════════════════════════════
// Stream NFT Contract
// ═══════════════════════════════════════════════════════════════════════════════

/// Wraps a stream receiver role as a Stellar custom asset (XLM-like token).
/// Transferring this token transfers the right to receive future stream payments.
#[contract]
pub struct StreamNftContract;

#[contractimpl]
impl StreamNftContract {
    /// Deploy the NFT contract for a stream. Called by the stream contract
    /// during `update_recipient` or initial setup.
    ///
    /// # Arguments
    /// * `admin` - The deployer (stream contract or factory).
    /// * `stream_contract` - The stream contract address this NFT wraps.
    pub fn initialize(
        env: Env,
        admin: Address,
        stream_contract: Address,
    ) -> Result<(), StreamNftError> {
        storage::set_admin(&env, &admin);
        storage::set_stream_contract(&env, &stream_contract);

        Ok(())
    }

    /// Mint an NFT representing the receiver role of a stream.
    ///
    /// # Arguments
    /// * `stream` - The stream contract address.
    /// * `owner` - The initial owner (current receiver).
    ///
    /// # Expected behavior
    /// 1. Validate stream_contract matches.
    /// 2. Validate caller is admin.
    /// 3. Generate token_id from stream address.
    /// 4. Set owner.
    /// 5. Emit `nft_minted`.
    /// 6. Return token_id.
    pub fn mint(_env: Env, _stream: Address, _owner: Address) -> Result<u32, StreamNftError> {
        // TODO: Validate stream matches stored stream_contract
        //   Generate deterministic token_id from stream address
        //   Store owner, emit mint event
        todo!("mint stream NFT")
    }

    /// Transfer NFT to a new address. The new owner becomes the stream
    /// receiver; the previous owner loses their receiver status.
    ///
    /// # Arguments
    /// * `from` - Current owner.
    /// * `to` - New owner.
    /// * `token_id` - The NFT token ID.
    pub fn transfer(
        _env: Env,
        _from: Address,
        _to: Address,
        _token_id: u32,
    ) -> Result<(), StreamNftError> {
        // TODO: Validate from == current owner
        //   Update owner to `to`
        //   Call stream contract update_recipient (cross-contract)
        //   Emit transfer event
        todo!("transfer stream NFT")
    }

    /// Burn the NFT, revoking the receiver role.
    ///
    /// # Expected behavior
    /// 1. Validate caller is owner.
    /// 2. Clear owner from storage.
    /// 3. Emit `nft_burned`.
    pub fn burn(_env: Env, _owner: Address, _token_id: u32) -> Result<(), StreamNftError> {
        // TODO: Validate owner, clear storage, emit burn
        todo!("burn stream NFT")
    }

    /// Read-only: get current owner of an NFT.
    pub fn owner_of(_env: Env, _token_id: u32) -> Result<Address, StreamNftError> {
        // TODO: Derive token_id -> owner
        todo!("get NFT owner")
    }

    /// Read-only: get the stream contract this NFT wraps.
    pub fn get_stream_contract(env: Env) -> Result<Address, StreamNftError> {
        Ok(storage::get_stream_contract(&env))
    }
}

mod test;
