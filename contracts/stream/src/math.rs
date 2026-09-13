use trickle_common::StreamStatus;

use super::storage::StreamConfig;
use super::StreamError;

/// Calculate the current claimable balance for a stream.
///
/// This is pure math — no `env` needed, no storage access.
/// Callers pass in the current config and the current ledger time.
///
/// # Pause semantics
/// While paused, accrual is frozen at the pause timestamp: the effective
/// time is `min(paused_at, current_time)`, so paused time never accrues.
/// On resume, `last_update_time` is reset to the resume time, excluding the
/// entire paused window from accrual.
///
/// # Formula
/// ```text
/// effective  = if paused { min(paused_at, current_time) } else { current_time }
/// elapsed    = effective - last_update_time
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
    let effective_time = match config.paused_at {
        Some(paused_at) => paused_at.min(current_time),
        None => current_time,
    };
    let elapsed = effective_time.saturating_sub(config.last_update_time);
    let accrued = config.flow_rate * elapsed as i128;
    let remaining = config.total_amount - config.withdrawn_amount;
    accrued.min(remaining)
}

/// The ledger time up to which the recipient's accrual is currently counted.
///
/// While paused this is the pause timestamp; otherwise it is the current ledger time.
/// Used when finalizing state during a withdraw so the checkpoint advances to the
/// frozen time rather than drifting through a paused window.
pub fn effective_time(config: &StreamConfig, current_time: u64) -> u64 {
    match config.paused_at {
        Some(paused_at) => paused_at.min(current_time),
        None => current_time,
    }
}

/// Calculate the per-second flow rate from total amount and duration.
///
/// Validates that both inputs are non-zero and performs integer division.
/// Note: integer division truncates. For example, 10_000 / 300 = 33,
/// meaning 1 token per stream will be "dust" that never streams.
/// A future enhancement could track remainder dust separately.
#[allow(dead_code)] // factory/multistream compute flow rates; kept as the shared definition
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
/// This is called by `storage::config_to_info`. Status is stored explicitly
/// and mutated by pause/resume/cancel; the only derived override is the
/// terminal `Completed` flag once the full escrow has been withdrawn.
///
/// # Ordering guarantee
/// The `Completed` override is gated on `status == Active`, so a stream that
/// has been cancelled can never be reported as `Completed` — even during the
/// transaction in which `cancel` sets `withdrawn_amount = total_amount`.
pub fn derive_status(config: &StreamConfig) -> StreamStatus {
    match config.status {
        StreamStatus::Cancelled => StreamStatus::Cancelled,
        StreamStatus::Active if config.withdrawn_amount >= config.total_amount => {
            StreamStatus::Completed
        }
        status => status,
    }
}
