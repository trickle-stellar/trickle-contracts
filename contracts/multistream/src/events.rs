use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a recipient is added to the multistream.
pub fn recipient_added(env: &Env, recipient: &Address, weight: u32) {
    env.events()
        .publish((symbol_short!("rcpnt_add"), recipient), weight);
}

/// Emitted when a recipient is removed from the multistream.
pub fn recipient_removed(env: &Env, recipient: &Address) {
    env.events()
        .publish((symbol_short!("rcpnt_rm"),), recipient);
}

/// Emitted when a recipient withdraws their share.
pub fn multistream_withdrawn(env: &Env, recipient: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("ms_wd"), recipient), amount);
}

/// Emitted when the multistream is paused.
pub fn multistream_paused(env: &Env) {
    env.events().publish((symbol_short!("ms_pause"),), 0u32);
}

/// Emitted when the multistream is resumed.
pub fn multistream_resumed(env: &Env) {
    env.events().publish((symbol_short!("ms_resume"),), 0u32);
}

/// Emitted when the multistream is cancelled.
pub fn multistream_cancelled(env: &Env) {
    env.events().publish((symbol_short!("ms_cancel"),), 0u32);
}
