use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};
use trickle_common::StreamInfo;

/// Storage keys for the factory contract.
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// The admin address that controls factory settings.
    Admin,
    /// WASM hash of the deployed stream contract.
    /// Used by the deployer to create new stream instances.
    StreamWasmHash,
    /// Monotonically increasing stream ID counter.
    StreamCount,
    /// Cached stream metadata, keyed by stream ID.
    StreamInfo(u32),
    /// All stream IDs created by a given sender address.
    StreamsBySender(Address),
    /// All stream IDs receiving funds for a given recipient address.
    StreamsByRecipient(Address),
}

// ─── Read Helpers ────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("factory not initialized")
}

pub fn get_stream_wasm_hash(env: &Env) -> BytesN<32> {
    env.storage()
        .instance()
        .get(&DataKey::StreamWasmHash)
        .expect("factory not initialized")
}

pub fn get_stream_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::StreamCount)
        .unwrap_or(0)
}

pub fn get_stream_info(env: &Env, stream_id: u32) -> Option<StreamInfo> {
    env.storage()
        .instance()
        .get(&DataKey::StreamInfo(stream_id))
}

pub fn get_streams_by_sender(env: &Env, sender: &Address) -> Vec<u32> {
    env.storage()
        .instance()
        .get(&DataKey::StreamsBySender(sender.clone()))
        .unwrap_or_else(|| soroban_sdk::vec![env])
}

pub fn get_streams_by_recipient(env: &Env, recipient: &Address) -> Vec<u32> {
    env.storage()
        .instance()
        .get(&DataKey::StreamsByRecipient(recipient.clone()))
        .unwrap_or_else(|| soroban_sdk::vec![env])
}

// ─── Write Helpers ───────────────────────────────────────────────────────────

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn set_stream_wasm_hash(env: &Env, hash: &BytesN<32>) {
    env.storage().instance().set(&DataKey::StreamWasmHash, hash);
}

pub fn set_stream_count(env: &Env, count: &u32) {
    env.storage().instance().set(&DataKey::StreamCount, count);
}

pub fn set_stream_info(env: &Env, stream_id: u32, info: &StreamInfo) {
    env.storage()
        .instance()
        .set(&DataKey::StreamInfo(stream_id), info);
}

pub fn add_stream_for_sender(env: &Env, sender: &Address, stream_id: u32) {
    let mut streams = get_streams_by_sender(env, sender);
    streams.push_back(stream_id);
    env.storage()
        .instance()
        .set(&DataKey::StreamsBySender(sender.clone()), &streams);
}

pub fn add_stream_for_recipient(env: &Env, recipient: &Address, stream_id: u32) {
    let mut streams = get_streams_by_recipient(env, recipient);
    streams.push_back(stream_id);
    env.storage()
        .instance()
        .set(&DataKey::StreamsByRecipient(recipient.clone()), &streams);
}
