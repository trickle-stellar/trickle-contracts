/// Calculate a single recipient's share of accrued funds.
///
/// # Formula
/// ```text
/// share = (recipient_weight / total_weight) * accrued
/// claimable = share - already_withdrawn
/// ```
///
/// # Arguments
/// * `accrued` - Total tokens accrued by the multistream since last update.
/// * `recipient_weight` - This recipient's proportional weight.
/// * `total_weight` - Sum of all recipients' weights.
/// * `already_withdrawn` - How much this recipient has already claimed.
///
/// # Returns
/// The amount this recipient can withdraw now. Returns 0 if claimable <= 0.
pub fn calculate_recipient_share(
    accrued: i128,
    recipient_weight: u32,
    total_weight: u32,
    already_withdrawn: i128,
) -> i128 {
    if total_weight == 0 {
        return 0;
    }

    let share = accrued * recipient_weight as i128 / total_weight as i128;
    let claimable = share - already_withdrawn;
    claimable.max(0)
}
