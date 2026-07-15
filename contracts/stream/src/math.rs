use trickle_common::StreamStatus;

use super::storage::StreamConfig;
use super::StreamError;

/// Calculate the current claimable balance for a stream.
///
/// This is pure math — no `env` needed, no storage access.
/// Callers pass in the current config and the current ledger time.
///
/// # Formula
/// ```text
/// elapsed    = current_time - last_update_time
/// accrued    = flow_rate * elapsed
/// remaining  = total_amount - withdrawn_amount
/// claimable  = min(accrued, remaining)
/// ```
///
/// # Overflow Safety
/// - `elapsed` uses `saturating_sub` (underflows to 0 if time goes backwards)
/// - `accrued` can overflow for very large flow_rate * elapsed — callers
///   should validate flow_rate at creation time to prevent this
/// - `.min(remaining)` caps at the escrow boundary
pub fn calculate_claimable(config: &StreamConfig, current_time: u64) -> i128 {
    let elapsed = current_time.saturating_sub(config.last_update_time);
    let accrued = config.flow_rate * elapsed as i128;
    let remaining = config.total_amount - config.withdrawn_amount;
    accrued.min(remaining)
}

/// Calculate the per-second flow rate from total amount and duration.
///
/// Validates that both inputs are non-zero and performs integer division.
/// Note: integer division truncates. For example, 10_000 / 300 = 33,
/// meaning 1 token per stream will be "dust" that never streams.
/// A future enhancement could track remainder dust separately.
pub fn calculate_flow_rate(amount: i128, duration: u32) -> Result<i128, StreamError> {
    if amount <= 0 {
        return Err(StreamError::ZeroAmount);
    }
    if duration == 0 {
        return Err(StreamError::InvalidFlowRate);
    }
    Ok(amount / duration as i128)
}

/// Calculate remaining unstreamed tokens.
pub fn calculate_remaining(config: &StreamConfig) -> i128 {
    config.total_amount - config.withdrawn_amount
}

/// Derive the current stream status from config state.
///
/// This is called by `storage::config_to_info` to determine status
/// without storing it explicitly — status is derived from state.
///
/// TODO: For the initial skeleton, status is stored explicitly in config.
/// A future enhancement could derive it purely from amounts and time.
pub fn derive_status(_config: &StreamConfig) -> StreamStatus {
    // For now, the status is stored in the config and set by
    // pause/resume/cancel operations. This function exists as a
    // placeholder for derived-status logic.
    StreamStatus::Active
}
