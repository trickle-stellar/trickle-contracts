use soroban_sdk::{contracttype, Address, Env};
use trickle_common::StreamInfo;

/// Storage keys specific to a single stream contract instance.
///
/// Each variant maps to a storage slot in instance storage.
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Whether this stream contract has been initialized.
    Initialized,
    /// The stream configuration — all mutable state for this stream.
    Config,
    /// The authorized stream-nft contract address.
    /// Only this address may call `update_recipient`.
    NftContract,
}

/// Internal mutable state for a single stream.
///
/// This is the raw storage type. Callers never see this directly —
/// `get_info` converts it to `StreamInfo` from trickle-common.
#[derive(Clone, Debug)]
#[contracttype]
pub struct StreamConfig {
    /// The factory contract that deployed this stream.
    pub factory: Address,
    /// Address that funded the stream and controls pause/cancel.
    pub sender: Address,
    /// Address that can withdraw accrued funds at any time.
    pub recipient: Address,
    /// Contract address of the Stellar token being streamed.
    pub asset: Address,
    /// Token units streamed per second (total_amount / duration).
    pub flow_rate: i128,
    /// Total tokens deposited into escrow for this stream.
    pub total_amount: i128,
    /// Total tokens the recipient has withdrawn so far.
    pub withdrawn_amount: i128,
    /// Ledger timestamp (Sec) when the stream started flowing.
    pub start_time: u64,
    /// Ledger timestamp (Sec) of the last state change
    /// (creation, withdrawal, resume).
    pub last_update_time: u64,
}

// ─── Read Helpers ────────────────────────────────────────────────────────────

/// Read the stream config from instance storage.
///
/// # Panics
/// Panics if the stream has not been initialized.
pub fn get_config(env: &Env) -> StreamConfig {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .expect("stream not initialized")
}

/// Check whether this stream contract has been initialized.
pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .instance()
        .get::<_, bool>(&DataKey::Initialized)
        .unwrap_or(false)
}

/// Read the authorized NFT contract address, if one has been set.
pub fn get_nft_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::NftContract)
}

// ─── Write Helpers ───────────────────────────────────────────────────────────

/// Store the stream config in instance storage.
pub fn set_config(env: &Env, config: &StreamConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

/// Mark this stream contract as initialized.
pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&DataKey::Initialized, &true);
}

/// Store the authorized NFT contract address.
pub fn set_nft_contract(env: &Env, address: &Address) {
    env.storage().instance().set(&DataKey::NftContract, address);
}

// ─── Conversion ──────────────────────────────────────────────────────────────

/// Convert internal StreamConfig to the public StreamInfo type.
pub fn config_to_info(config: &StreamConfig) -> StreamInfo {
    StreamInfo {
        sender: config.sender.clone(),
        recipient: config.recipient.clone(),
        asset: config.asset.clone(),
        flow_rate: config.flow_rate,
        total_amount: config.total_amount,
        withdrawn_amount: config.withdrawn_amount,
        start_time: config.start_time,
        last_update_time: config.last_update_time,
        status: super::math::derive_status(config),
    }
}
