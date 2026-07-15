use soroban_sdk::{contracttype, Address, Env, String};

/// Storage keys for the stream NFT contract.
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Admin who deployed this NFT contract.
    Admin,
    /// Stream contract address this NFT wraps.
    StreamContract,
    /// Token symbol (e.g., "sNFT").
    Symbol,
    /// Metadata URI (optional).
    MetadataUri,
    /// The current owner of this NFT.
    Owner,
}

// ─── Read Helpers ────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("stream-nft not initialized")
}

pub fn get_stream_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::StreamContract)
        .expect("stream contract not set")
}

pub fn get_owner(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Owner)
}

pub fn get_metadata_uri(env: &Env) -> Option<String> {
    env.storage().instance().get(&DataKey::MetadataUri)
}

// ─── Write Helpers ───────────────────────────────────────────────────────────

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn set_stream_contract(env: &Env, stream: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::StreamContract, stream);
}

pub fn set_owner(env: &Env, owner: &Address) {
    env.storage().instance().set(&DataKey::Owner, owner);
}

pub fn set_metadata_uri(env: &Env, uri: &String) {
    env.storage().instance().set(&DataKey::MetadataUri, uri);
}
