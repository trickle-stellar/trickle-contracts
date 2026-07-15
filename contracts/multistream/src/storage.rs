use soroban_sdk::{contracttype, Address, Env};
use trickle_common::RecipientInfo;

/// Storage keys for the multistream contract.
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Whether this multistream has been initialized.
    Initialized,
    /// The multistream configuration.
    Config,
    /// Recipient data, keyed by address.
    Recipient(Address),
    /// Sum of all recipient weights.
    TotalWeight,
    /// Amount already withdrawn by a recipient.
    Withdrawn(Address),
}

/// Internal configuration for a multi-recipient stream.
#[derive(Clone, Debug)]
#[contracttype]
pub struct MultiStreamConfig {
    pub sender: Address,
    pub asset: Address,
    pub total_amount: i128,
    pub duration: u32,
    pub flow_rate: i128,
    pub start_time: u64,
    pub last_update_time: u64,
}

// ─── Read Helpers ────────────────────────────────────────────────────────────

pub fn get_config(env: &Env) -> MultiStreamConfig {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .expect("multistream not initialized")
}

pub fn get_recipient_info(env: &Env, recipient: &Address) -> Option<RecipientInfo> {
    env.storage()
        .instance()
        .get(&DataKey::Recipient(recipient.clone()))
}

pub fn get_total_weight(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::TotalWeight)
        .unwrap_or(0)
}

pub fn get_withdrawn(env: &Env, recipient: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::Withdrawn(recipient.clone()))
        .unwrap_or(0)
}

// ─── Write Helpers ───────────────────────────────────────────────────────────

pub fn set_config(env: &Env, config: &MultiStreamConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn set_recipient_info(env: &Env, recipient: &Address, info: &RecipientInfo) {
    env.storage()
        .instance()
        .set(&DataKey::Recipient(recipient.clone()), info);
}

pub fn remove_recipient_info(env: &Env, recipient: &Address) {
    env.storage()
        .instance()
        .remove(&DataKey::Recipient(recipient.clone()));
}

pub fn set_total_weight(env: &Env, weight: &u32) {
    env.storage().instance().set(&DataKey::TotalWeight, weight);
}

pub fn set_withdrawn(env: &Env, recipient: &Address, amount: &i128) {
    env.storage()
        .instance()
        .set(&DataKey::Withdrawn(recipient.clone()), amount);
}
