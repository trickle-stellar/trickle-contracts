use trickle_common::VestingError;

/// Calculate the vested amount at a given timestamp.
///
/// # Vesting Schedule
/// 1. Before cliff: nothing vested.
/// 2. After cliff: linear vesting from cliff to end.
///
/// ```text
/// vested = total_amount * (elapsed - cliff_duration) / (vesting_duration - cliff_duration)
/// ```
///
/// # Arguments
/// * `total_amount` - Total tokens in the vesting contract.
/// * `elapsed` - Seconds since vesting start.
/// * `cliff_duration` - Cliff period in seconds.
/// * `vesting_duration` - Total vesting duration in seconds.
///
/// # Returns
/// The total vested amount. Returns 0 if before cliff or if durations are invalid.
pub fn calculate_vested(
    total_amount: i128,
    elapsed: u64,
    cliff_duration: u64,
    vesting_duration: u64,
) -> Result<i128, VestingError> {
    if vesting_duration <= cliff_duration {
        return Err(VestingError::NothingToVest);
    }

    let elapsed = elapsed as i128;
    let cliff = cliff_duration as i128;
    let duration = vesting_duration as i128;

    if elapsed < cliff {
        return Ok(0);
    }

    let vested = total_amount * (elapsed - cliff) / (duration - cliff);

    // Cap at total_amount (shouldn't exceed, but safety check)
    Ok(vested.min(total_amount))
}

/// Calculate the claimable amount (vested - already claimed).
///
/// # Arguments
/// * `total_amount` - Total tokens in the vesting contract.
/// * `elapsed` - Seconds since vesting start.
/// * `cliff_duration` - Cliff period in seconds.
/// * `vesting_duration` - Total vesting duration in seconds.
/// * `already_claimed` - Tokens already claimed by beneficiary.
///
/// # Returns
/// The amount available to claim now.
pub fn calculate_claimable(
    total_amount: i128,
    elapsed: u64,
    cliff_duration: u64,
    vesting_duration: u64,
    already_claimed: i128,
) -> Result<i128, VestingError> {
    let vested = calculate_vested(total_amount, elapsed, cliff_duration, vesting_duration)?;
    let claimable = vested - already_claimed;
    Ok(claimable.max(0))
}
