use soroban_sdk::{contracttype, Address, Env};

/// Storage keys for the fee contract.
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Admin address that controls fee settings.
    Admin,
    /// Address where collected fees are sent on withdrawal.
    FeeRecipient,
    /// Fee rate in basis points (100 = 1%, 250 = 2.5%, 10000 = 100%).
    FeeRate,
    /// Accumulated fees waiting to be withdrawn by admin.
    AccumulatedFees,
}

// ─── Read Helpers ────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("fee contract not initialized")
}

pub fn get_fee_recipient(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::FeeRecipient)
        .expect("fee recipient not set")
}

pub fn get_fee_rate(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::FeeRate).unwrap_or(0)
}

pub fn get_accumulated_fees(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::AccumulatedFees)
        .unwrap_or(0)
}

// ─── Write Helpers ───────────────────────────────────────────────────────────

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn set_fee_recipient(env: &Env, recipient: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::FeeRecipient, recipient);
}

pub fn set_fee_rate(env: &Env, rate: &u32) {
    env.storage().instance().set(&DataKey::FeeRate, rate);
}

pub fn add_accumulated_fees(env: &Env, amount: i128) {
    let current = get_accumulated_fees(env);
    env.storage()
        .instance()
        .set(&DataKey::AccumulatedFees, &(current + amount));
}

pub fn reset_accumulated_fees(env: &Env) {
    env.storage()
        .instance()
        .set(&DataKey::AccumulatedFees, &0i128);
}
