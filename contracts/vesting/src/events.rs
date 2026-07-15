use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a vesting schedule is created.
pub fn vesting_initialized(env: &Env, beneficiary: &Address, total_amount: i128) {
    env.events()
        .publish((symbol_short!("vest_init"), beneficiary), total_amount);
}

/// Emitted when the beneficiary claims vested tokens.
pub fn vested_claimed(env: &Env, beneficiary: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("vest_clm"), beneficiary), amount);
}

/// Emitted when a revocable vesting is revoked (remaining unvested returned).
pub fn vesting_revoked(env: &Env, admin: &Address, returned: i128) {
    env.events()
        .publish((symbol_short!("vest_rvk"), admin), returned);
}
