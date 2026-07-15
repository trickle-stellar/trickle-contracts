use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when an NFT is minted (stream receiver is tokenized).
pub fn nft_minted(env: &Env, stream: &Address, owner: &Address, token_id: u32) {
    env.events()
        .publish((symbol_short!("nft_mint"), stream), (owner, token_id));
}

/// Emitted when an NFT is transferred to a new owner.
pub fn nft_transferred(env: &Env, from: &Address, to: &Address, token_id: u32) {
    env.events()
        .publish((symbol_short!("nft_xfer"),), (from, to, token_id));
}

/// Emitted when an NFT is burned (receiver role revoked).
pub fn nft_burned(env: &Env, owner: &Address, token_id: u32) {
    env.events()
        .publish((symbol_short!("nft_burn"),), (owner, token_id));
}
