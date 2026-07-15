use trickle_common::FeeError;

/// Calculate protocol fee as: `amount * fee_rate / 10_000`.
///
/// Fee rate is in basis points where 100 = 1%, 250 = 2.5%, 10000 = 100%.
///
/// # Overflow Safety
/// Uses `checked_mul` to prevent i128 overflow on large amounts.
/// Returns `FeeCalculationOverflow` if the intermediate product exceeds i128.
pub fn calculate_fee(amount: i128, fee_rate: u32) -> Result<i128, FeeError> {
    if fee_rate == 0 {
        return Ok(0);
    }

    let fee = amount
        .checked_mul(fee_rate as i128)
        .ok_or(FeeError::FeeCalculationOverflow)?
        / 10_000;

    Ok(fee)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_basic() {
        // 10_000 tokens * 250 basis points (2.5%) / 10_000 = 250
        assert_eq!(calculate_fee(10_000, 250).unwrap(), 250);
    }

    #[test]
    fn test_calculate_fee_zero_rate() {
        assert_eq!(calculate_fee(10_000, 0).unwrap(), 0);
    }

    #[test]
    fn test_calculate_fee_zero_amount() {
        assert_eq!(calculate_fee(0, 250).unwrap(), 0);
    }

    #[test]
    fn test_calculate_fee_one_percent() {
        // 1_000_000 * 100 / 10_000 = 10_000
        assert_eq!(calculate_fee(1_000_000, 100).unwrap(), 10_000);
    }
}
