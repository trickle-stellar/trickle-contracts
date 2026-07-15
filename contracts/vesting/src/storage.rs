use soroban_sdk::{contracttype, Address, Env};

/// Storage keys for the vesting contract.
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Admin / beneficiary who owns the vesting schedule.
    Admin,
    /// The token contract address.
    Asset,
    /// The vesting configuration.
    Config,
    /// Amount already claimed.
    Claimed,
}

/// Internal configuration for a vesting contract.
#[derive(Clone, Debug)]
#[contracttype]
pub struct VestingConfig {
    pub beneficiary: Address,
    pub asset: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub revocable: bool,
    pub revocation_time: Option<u64>,
}

// ─── Read Helpers ────────────────────────────────────────────────────────────

pub fn get_config(env: &Env) -> VestingConfig {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .expect("vesting not initialized")
}

pub fn get_claimed(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::Claimed).unwrap_or(0)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("vesting not initialized")
}

// ─── Write Helpers ───────────────────────────────────────────────────────────

pub fn set_config(env: &Env, config: &VestingConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn set_claimed(env: &Env, amount: &i128) {
    env.storage().instance().set(&DataKey::Claimed, amount);
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}
