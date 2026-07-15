#![no_std]

use soroban_sdk::{contracterror, contracttype, Address};

// ═══════════════════════════════════════════════════════════════════════════════
// Stream Types — used by: stream, factory
// ═══════════════════════════════════════════════════════════════════════════════

/// Status of a payment stream through its lifecycle.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum StreamStatus {
    Active,
    Paused,
    Cancelled,
    Completed,
}

/// Public-facing stream metadata returned by read-only queries.
///
/// This is the type callers receive from `get_info` / `get_stream`.
/// Internal mutable state (withdrawn_amount, last_update_time) is
/// stored in the stream contract's private `StreamConfig` struct.
#[derive(Clone, Debug)]
#[contracttype]
pub struct StreamInfo {
    pub sender: Address,
    pub recipient: Address,
    pub asset: Address,
    pub flow_rate: i128,
    pub total_amount: i128,
    pub withdrawn_amount: i128,
    pub start_time: u64,
    pub last_update_time: u64,
    pub status: StreamStatus,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stream Errors
// ═══════════════════════════════════════════════════════════════════════════════

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StreamError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    StreamNotFound = 4,
    InsufficientBalance = 5,
    StreamNotActive = 6,
    StreamAlreadyCompleted = 7,
    InvalidFlowRate = 8,
    ZeroAmount = 9,
    NothingToWithdraw = 10,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Multistream Types — used by: multistream
// ═══════════════════════════════════════════════════════════════════════════════

/// A recipient in a multi-recipient stream with their proportional weight.
#[derive(Clone, Debug)]
#[contracttype]
pub struct RecipientInfo {
    pub address: Address,
    pub weight: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MultistreamError {
    StreamNotFound = 1,
    Unauthorized = 2,
    NoRecipients = 3,
    InvalidWeight = 4,
    StreamNotActive = 5,
    NothingToWithdraw = 6,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Vesting Types — used by: vesting
// ═══════════════════════════════════════════════════════════════════════════════

/// Status of a vesting schedule.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum VestingStatus {
    NotStarted,
    Vesting,
    Completed,
    Revoked,
}

/// Public-facing vesting schedule metadata.
#[derive(Clone, Debug)]
#[contracttype]
pub struct VestingInfo {
    pub beneficiary: Address,
    pub asset: Address,
    pub total_amount: i128,
    pub vested_amount: i128,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub status: VestingStatus,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VestingError {
    VestingNotFound = 1,
    Unauthorized = 2,
    NothingToVest = 3,
    VestingAlreadyCompleted = 4,
    CliffNotReached = 5,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fee Errors — used by: fees
// ═══════════════════════════════════════════════════════════════════════════════

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FeeError {
    NotInitialized = 1,
    FeeNotConfigured = 2,
    FeeCalculationOverflow = 3,
    InsufficientFeeBalance = 4,
    Unauthorized = 5,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stream NFT Errors — used by: stream-nft
// ═══════════════════════════════════════════════════════════════════════════════

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StreamNftError {
    StreamNotFound = 1,
    Unauthorized = 2,
    NotOwner = 3,
    NftAlreadyMinted = 4,
}
