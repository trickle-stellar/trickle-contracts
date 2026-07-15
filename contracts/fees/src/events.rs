use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a fee is collected on stream creation or withdrawal.
pub fn fee_collected(env: &Env, payer: &Address, amount: i128, fee: i128) {
    env.events()
        .publish((symbol_short!("fee"), payer), (amount, fee));
}

/// Emitted when the admin withdraws accumulated fees.
pub fn fees_withdrawn(env: &Env, admin: &Address, amount: i128, recipient: &Address) {
    env.events()
        .publish((symbol_short!("fee_wd"), admin), (amount, recipient));
}

/// Emitted when the fee rate is updated.
pub fn fee_rate_updated(env: &Env, old_rate: u32, new_rate: u32) {
    env.events()
        .publish((symbol_short!("fee_rate"),), (old_rate, new_rate));
}
